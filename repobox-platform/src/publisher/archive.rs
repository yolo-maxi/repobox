//! Uploaded image archives: `docker save` output (optionally gzipped), which
//! is a tar holding `manifest.json` (Docker's format) and/or `index.json`
//! (an OCI image layout, which is also what `podman save --format
//! oci-archive` and `docker buildx build --output type=oci` produce).
//!
//! [`rename`] streams an archive into a copy whose image names are replaced
//! by the platform's own (`repobox-pub/<app>:<release>`), hashing the input
//! as it goes. Layers and configs are copied byte for byte, so their
//! digests are untouched; only the name records change. `docker load` of the
//! copy therefore creates exactly one, platform-named image — whatever tags
//! the uploader's archive carried are never applied on the host.

use std::io::{Read, Write};

use sha2::{Digest, Sha256};

/// Largest accepted upload (compressed or not).
pub const MAX_UPLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
/// Largest name record read into memory.
const MAX_META_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveError {
    pub code: &'static str,
    pub message: String,
}

fn err(code: &'static str, message: impl Into<String>) -> ArchiveError {
    ArchiveError {
        code,
        message: message.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// `sha256:<hex>` of the uploaded bytes.
    pub sha256: String,
    pub bytes: u64,
    /// `docker-save`, `oci-layout`, with `+gzip` when compressed.
    pub format: String,
}

/// Does this look like an image archive (tar or gzip)? `head` is the first
/// bytes of the upload (at least 262 for a tar).
pub fn sniff(head: &[u8]) -> Option<bool> {
    if head.starts_with(&[0x1f, 0x8b]) {
        Some(true)
    } else if head.len() >= 262 && &head[257..262] == b"ustar" {
        Some(false)
    } else {
        None
    }
}

struct Hashing<R> {
    inner: R,
    h: Sha256,
    n: u64,
}

impl<R: Read> Read for Hashing<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.h.update(&buf[..n]);
        self.n += n as u64;
        Ok(n)
    }
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Rewrite `src` into `out` with every image name replaced by `image`
/// (`repo:tag`). Refuses archives that hold other than exactly one image.
pub fn rename<R: Read, W: Write>(src: R, out: W, image: &str) -> Result<Summary, ArchiveError> {
    let mut hashing = Hashing {
        inner: src,
        h: Sha256::new(),
        n: 0,
    };
    let mut magic = [0u8; 2];
    hashing
        .read_exact(&mut magic)
        .map_err(|_| err("bad_archive", "the upload is empty"))?;
    let gzip = magic == [0x1f, 0x8b];
    let chained = std::io::Cursor::new(magic).chain(hashing);
    let (chained, format) = if gzip {
        let (gz, f) = rewrite(flate2::read::GzDecoder::new(chained), out, image)?;
        (gz.into_inner(), format!("{f}+gzip"))
    } else {
        rewrite(chained, out, image)?
    };
    let (_, mut hashing) = chained.into_inner();
    std::io::copy(&mut hashing, &mut std::io::sink())
        .map_err(|e| err("bad_archive", format!("the upload ends early ({e})")))?;
    Ok(Summary {
        sha256: format!("sha256:{}", hex(&hashing.h.finalize())),
        bytes: hashing.n,
        format,
    })
}

fn rewrite<R: Read, W: Write>(src: R, out: W, image: &str) -> Result<(R, String), ArchiveError> {
    let (repo, tag) = image
        .rsplit_once(':')
        .ok_or_else(|| err("internal", "image name needs a tag"))?;
    let mut ar = tar::Archive::new(src);
    let mut b = tar::Builder::new(out);
    let bad = |e: std::io::Error| err("bad_archive", format!("not a readable image archive ({e})"));
    let (mut docker_manifest, mut oci_index) = (false, false);
    for entry in ar.entries().map_err(bad)? {
        let mut e = entry.map_err(bad)?;
        let path = e.path().map_err(bad)?.to_string_lossy().into_owned();
        let name = path.trim_start_matches("./").to_string();
        let mut h = e.header().clone();
        use tar::EntryType as T;
        match (name.as_str(), e.header().entry_type()) {
            ("repositories", _) => continue,
            ("manifest.json", T::Regular) | ("index.json", T::Regular) => {
                if e.size() > MAX_META_BYTES {
                    return Err(err("bad_archive", format!("{name} is too large")));
                }
                let mut v: serde_json::Value = {
                    let mut s = Vec::new();
                    e.read_to_end(&mut s).map_err(bad)?;
                    serde_json::from_slice(&s)
                        .map_err(|_| err("bad_archive", format!("{name} is not JSON")))?
                };
                if name == "manifest.json" {
                    docker_manifest = true;
                    let list = v
                        .as_array_mut()
                        .ok_or_else(|| err("bad_archive", "manifest.json is not a list"))?;
                    if list.len() != 1 {
                        return Err(err(
                            "not_one_image",
                            format!(
                                "the archive holds {} images; save exactly one (docker save <one image>)",
                                list.len()
                            ),
                        ));
                    }
                    list[0]["RepoTags"] = serde_json::json!([image]);
                } else {
                    oci_index = true;
                    let list = v["manifests"]
                        .as_array_mut()
                        .ok_or_else(|| err("bad_archive", "index.json has no manifests"))?;
                    if list.len() != 1 {
                        return Err(err(
                            "not_one_image",
                            format!(
                                "the archive holds {} images; save exactly one (and build with --provenance=false when using buildx)",
                                list.len()
                            ),
                        ));
                    }
                    let d = &mut list[0];
                    if !d["annotations"].is_object() {
                        d["annotations"] = serde_json::json!({});
                    }
                    let a = d["annotations"].as_object_mut().unwrap();
                    a.insert(
                        "io.containerd.image.name".into(),
                        format!("docker.io/{repo}:{tag}").into(),
                    );
                    a.insert("org.opencontainers.image.ref.name".into(), tag.into());
                }
                let bytes = serde_json::to_vec(&v).map_err(|e| err("internal", e.to_string()))?;
                h.set_size(bytes.len() as u64);
                b.append_data(&mut h, &name, &bytes[..]).map_err(bad)?;
            }
            (_, T::Regular | T::Continuous) => {
                b.append_data(&mut h, &name, &mut e).map_err(bad)?;
            }
            (_, T::Directory) => {
                b.append_data(&mut h, &name, std::io::empty())
                    .map_err(bad)?;
            }
            (_, T::Symlink | T::Link) => {
                let target = e
                    .link_name()
                    .map_err(bad)?
                    .ok_or_else(|| err("bad_archive", format!("{name}: link without target")))?
                    .into_owned();
                b.append_link(&mut h, &name, target).map_err(bad)?;
            }
            (_, T::XGlobalHeader | T::XHeader | T::GNULongName | T::GNULongLink) => {}
            _ => {
                return Err(err(
                    "bad_archive",
                    format!("{name}: unexpected entry type in an image archive"),
                ));
            }
        }
    }
    if !docker_manifest && !oci_index {
        return Err(err(
            "bad_archive",
            "not an image archive (no manifest.json or index.json); upload the output of `docker save`",
        ));
    }
    b.finish().map_err(bad)?;
    let format = if docker_manifest {
        "docker-save"
    } else {
        "oci-layout"
    };
    Ok((ar.into_inner(), format.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive(files: &[(&str, &str)]) -> Vec<u8> {
        let mut b = tar::Builder::new(Vec::new());
        for (p, c) in files {
            let mut h = tar::Header::new_ustar();
            h.set_size(c.len() as u64);
            h.set_mode(0o644);
            b.append_data(&mut h, p, c.as_bytes()).unwrap();
        }
        b.into_inner().unwrap()
    }

    fn read(tar_bytes: &[u8]) -> std::collections::BTreeMap<String, String> {
        let mut ar = tar::Archive::new(tar_bytes);
        ar.entries()
            .unwrap()
            .map(|e| {
                let mut e = e.unwrap();
                let p = e.path().unwrap().to_string_lossy().into_owned();
                let mut s = String::new();
                e.read_to_string(&mut s).unwrap();
                (p, s)
            })
            .collect()
    }

    const SAVE: &[(&str, &str)] = &[
        ("blobs/sha256/aa", "layer-bytes"),
        (
            "index.json",
            r#"{"schemaVersion":2,"manifests":[{"digest":"sha256:bb","annotations":{"io.containerd.image.name":"docker.io/library/jellyfin:latest","org.opencontainers.image.ref.name":"latest"}}]}"#,
        ),
        (
            "manifest.json",
            r#"[{"Config":"blobs/sha256/cc","RepoTags":["jellyfin/jellyfin:latest"],"Layers":["blobs/sha256/aa"]}]"#,
        ),
        ("oci-layout", r#"{"imageLayoutVersion":"1.0.0"}"#),
        ("repositories", r#"{"jellyfin/jellyfin":{"latest":"cc"}}"#),
    ];

    #[test]
    fn names_are_replaced_and_layers_untouched() {
        let src = archive(SAVE);
        let mut out = Vec::new();
        let s = rename(&src[..], &mut out, "repobox-pub/trip:rel-1").unwrap();
        assert_eq!(s.format, "docker-save");
        assert_eq!(s.bytes, src.len() as u64);
        assert_eq!(s.sha256, format!("sha256:{}", hex(&Sha256::digest(&src))));
        let files = read(&out);
        assert!(!files.contains_key("repositories"));
        assert_eq!(files["blobs/sha256/aa"], "layer-bytes");
        assert!(files["manifest.json"].contains(r#""RepoTags":["repobox-pub/trip:rel-1"]"#));
        assert!(!files["manifest.json"].contains("jellyfin"));
        assert!(!files["index.json"].contains("jellyfin"));
        assert!(files["index.json"].contains("docker.io/repobox-pub/trip:rel-1"));

        // gzip and OCI-only (podman oci-archive / buildx type=oci) archives
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(&src).unwrap();
        let gz = gz.finish().unwrap();
        let mut out = Vec::new();
        let s = rename(&gz[..], &mut out, "repobox-pub/trip:rel-2").unwrap();
        assert_eq!(s.format, "docker-save+gzip");
        assert_eq!(s.bytes, gz.len() as u64);
        let oci: Vec<_> = SAVE
            .iter()
            .copied()
            .filter(|(p, _)| *p != "manifest.json" && *p != "repositories")
            .collect();
        let mut out = Vec::new();
        assert_eq!(
            rename(&archive(&oci)[..], &mut out, "repobox-pub/trip:rel-3")
                .unwrap()
                .format,
            "oci-layout"
        );
        assert!(read(&out)["index.json"].contains("docker.io/repobox-pub/trip:rel-3"));
    }

    #[test]
    fn refusals() {
        let code = |a: &[u8]| {
            rename(a, &mut Vec::new(), "repobox-pub/x:r")
                .unwrap_err()
                .code
        };
        assert_eq!(code(&archive(&[("hello.txt", "hi")])), "bad_archive");
        assert_eq!(
            code(&archive(&[(
                "manifest.json",
                r#"[{"RepoTags":["a:1"]},{"RepoTags":["b:1"]}]"#
            )])),
            "not_one_image"
        );
        assert_eq!(
            code(&archive(&[(
                "index.json",
                r#"{"manifests":[{"digest":"a"},{"digest":"b"}]}"#
            )])),
            "not_one_image"
        );
        assert_eq!(code(b""), "bad_archive");
        assert_eq!(sniff(&[0x1f, 0x8b, 8]), Some(true));
        assert_eq!(sniff(&archive(SAVE)[..512]), Some(false));
        assert_eq!(sniff(b"PK\x03\x04"), None);
    }
}
