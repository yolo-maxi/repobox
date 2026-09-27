//! The release manifest: the small JSON document sent next to an uploaded
//! Docker image archive (`docker save`).
//!
//! Minimal:
//!
//! ```json
//! {"name": "trip-planner", "title": "Trip planner"}
//! ```
//!
//! Full:
//!
//! ```json
//! {"name": "trip-planner", "title": "Trip planner", "description": "…",
//!  "version": "1.4.0",
//!  "runtime": {"port": 3000, "health_path": "/healthz", "memory_mb": 512,
//!              "env": {"LOG_LEVEL": "info"}},
//!  "ai": true,
//!  "provenance": {"repository": "https://github.com/acme/trip-planner", "commit": "9f2c1e7", "note": "built in CI"}}
//! ```
//!
//! `runtime.port` is the port the app listens on inside its container
//! (also passed as `$PORT`); unset, the image's single EXPOSEd TCP port is
//! used, else 8080. `provenance` is recorded only; nothing is fetched.
//!
//! Publishers are trusted first-party agents, so validation is about a
//! well-formed, unambiguous request (and never letting a value become a
//! command-line option or a route). Nothing from the manifest reaches the
//! generated Caddy route except the validated DNS label `name`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::validate_app_name;

pub const MAX_MANIFEST_BYTES: usize = 32 * 1024;
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_MEMORY_MB: u32 = 512;
pub const MAX_MEMORY_MB: u32 = 1024;
pub const MIN_MEMORY_MB: u32 = 64;
pub const MAX_ENV: usize = 64;
/// Env names the platform sets itself.
pub const PLATFORM_ENV: &[&str] = &["PORT"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// The publisher's own label for this build.
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub runtime: Runtime,
    /// AI capability for a *new* app (default on). An update keeps the
    /// app's current AI policy.
    pub ai: bool,
    #[serde(default)]
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Runtime {
    /// Container port; None: the image's single EXPOSEd TCP port, else 8080.
    #[serde(default)]
    pub port: Option<u16>,
    pub health_path: String,
    pub memory_mb: u32,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            port: None,
            health_path: "/".into(),
            memory_mb: DEFAULT_MEMORY_MB,
            env: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

fn bad(message: impl Into<String>) -> ManifestError {
    ManifestError {
        code: "invalid_manifest",
        message: message.into(),
    }
}

fn only(
    obj: &serde_json::Map<String, Value>,
    what: &str,
    allowed: &[&str],
) -> Result<(), ManifestError> {
    for k in obj.keys() {
        if !allowed.contains(&k.as_str()) {
            return Err(bad(format!(
                "unknown field '{what}{k}' (allowed: {})",
                allowed.join(", ")
            )));
        }
    }
    Ok(())
}

fn text(
    v: &Value,
    key: &str,
    label: &str,
    max: usize,
    required: bool,
) -> Result<String, ManifestError> {
    match v.get(key) {
        None | Some(Value::Null) if !required => Ok(String::new()),
        None | Some(Value::Null) => Err(bad(format!("'{label}' is required"))),
        Some(Value::String(s)) => {
            let s = s.trim();
            if required && s.is_empty() {
                return Err(bad(format!("'{label}' may not be empty")));
            }
            if s.chars().count() > max {
                return Err(bad(format!("'{label}' must be at most {max} characters")));
            }
            if s.chars().any(|c| c.is_control() && c != '\n') {
                return Err(bad(format!("'{label}' may not contain control characters")));
            }
            Ok(s.to_string())
        }
        Some(_) => Err(bad(format!("'{label}' must be a string"))),
    }
}

pub fn parse(bytes: &[u8]) -> Result<Manifest, ManifestError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(bad(format!(
            "the manifest must be at most {MAX_MANIFEST_BYTES} bytes"
        )));
    }
    let v: Value = serde_json::from_slice(bytes)
        .map_err(|e| bad(format!("the manifest is not valid JSON ({e})")))?;
    from_value(&v)
}

pub fn from_value(v: &Value) -> Result<Manifest, ManifestError> {
    let obj = v
        .as_object()
        .ok_or_else(|| bad("the manifest must be a JSON object"))?;
    only(
        obj,
        "",
        &[
            "name",
            "title",
            "description",
            "version",
            "runtime",
            "ai",
            "provenance",
        ],
    )?;
    let name = text(v, "name", "name", 63, true)?;
    validate_app_name(&name).map_err(|e| bad(format!("'name': {e}")))?;
    let title = text(v, "title", "title", 80, true)?;
    let description = text(v, "description", "description", 500, false)?;
    let version = text(v, "version", "version", 64, false)?;
    let ai = match obj.get("ai") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(bad("'ai' must be a boolean")),
    };

    let mut runtime = Runtime::default();
    match obj.get("runtime") {
        None | Some(Value::Null) => {}
        Some(r @ Value::Object(m)) => {
            only(m, "runtime.", &["port", "health_path", "memory_mb", "env"])?;
            if let Some(p) = m.get("port").filter(|p| !p.is_null()) {
                runtime.port = Some(
                    p.as_u64()
                        .filter(|p| (1..=65535).contains(p))
                        .ok_or_else(|| bad("'runtime.port' must be 1-65535"))?
                        as u16,
                );
            }
            let hp = text(r, "health_path", "runtime.health_path", 200, false)?;
            if !hp.is_empty() {
                if !hp.starts_with('/') || hp.contains(char::is_whitespace) || hp.starts_with("//")
                {
                    return Err(bad("'runtime.health_path' must be a path like /healthz"));
                }
                runtime.health_path = hp;
            }
            if let Some(mb) = m.get("memory_mb").filter(|x| !x.is_null()) {
                runtime.memory_mb = mb
                    .as_u64()
                    .filter(|x| (MIN_MEMORY_MB as u64..=MAX_MEMORY_MB as u64).contains(x))
                    .ok_or_else(|| {
                        bad(format!(
                            "'runtime.memory_mb' must be {MIN_MEMORY_MB}-{MAX_MEMORY_MB}"
                        ))
                    })? as u32;
            }
            match m.get("env") {
                None | Some(Value::Null) => {}
                Some(Value::Object(env)) => {
                    if env.len() > MAX_ENV {
                        return Err(bad(format!("at most {MAX_ENV} env variables")));
                    }
                    for (k, val) in env {
                        let key_ok = !k.is_empty()
                            && k.len() <= 64
                            && k.bytes()
                                .next()
                                .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                            && k.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
                        if !key_ok {
                            return Err(bad(format!(
                                "env name '{k}' must match [A-Za-z_][A-Za-z0-9_]*"
                            )));
                        }
                        if PLATFORM_ENV.contains(&k.as_str()) || k.starts_with("REPOBOX_") {
                            return Err(bad(format!("env '{k}' is set by the platform")));
                        }
                        let s = match val {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => {
                                return Err(bad(format!(
                                    "env '{k}' must be a string, number or boolean"
                                )));
                            }
                        };
                        if s.len() > 4096 || s.contains('\0') {
                            return Err(bad(format!("env '{k}' must be at most 4096 bytes")));
                        }
                        runtime.env.insert(k.clone(), s);
                    }
                }
                Some(_) => return Err(bad("'runtime.env' must be an object")),
            }
        }
        Some(_) => return Err(bad("'runtime' must be an object")),
    }

    let mut provenance = Provenance::default();
    match obj.get("provenance") {
        None | Some(Value::Null) => {}
        Some(p @ Value::Object(m)) => {
            only(m, "provenance.", &["repository", "commit", "note"])?;
            provenance.repository = text(p, "repository", "provenance.repository", 300, false)?;
            provenance.commit = text(p, "commit", "provenance.commit", 64, false)?;
            provenance.note = text(p, "note", "provenance.note", 300, false)?;
        }
        Some(_) => return Err(bad("'provenance' must be an object")),
    }

    Ok(Manifest {
        name,
        title,
        description,
        version,
        runtime,
        ai,
        provenance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_and_full() {
        let m = parse(br#"{"name":"trip","title":"Trip"}"#).unwrap();
        assert_eq!(m.name, "trip");
        assert!(m.ai);
        assert_eq!(m.runtime.port, None);
        assert_eq!(m.runtime.health_path, "/");
        assert_eq!(m.runtime.memory_mb, 512);
        let m = parse(
            br#"{"name":"trip","title":"Trip","version":"1.0.0+b1","ai":false,
            "runtime":{"port":3000,"health_path":"/healthz","memory_mb":256,"env":{"LOG_LEVEL":"info","WORKERS":2}},
            "provenance":{"repository":"https://github.com/acme/trip","commit":"abc1234","note":"CI"}}"#,
        )
        .unwrap();
        assert!(!m.ai);
        assert_eq!(m.runtime.port, Some(3000));
        assert_eq!(m.runtime.env["WORKERS"], "2");
        assert_eq!(m.provenance.commit, "abc1234");
        // The stored form reads back identically.
        let again = from_value(&serde_json::to_value(&m).unwrap()).unwrap();
        assert_eq!(again, m);
    }

    #[test]
    fn refusals() {
        for b in [
            &br#"{"name":"Trip","title":"T"}"#[..],
            br#"{"name":"auth","title":"T"}"#,
            br#"{"name":"a.b","title":"T"}"#,
            br#"{"name":"trip"}"#,
            br#"{"name":"trip","title":"T","image":"ghcr.io/a/b:1"}"#,
            br#"{"name":"trip","title":"T","target":"/etc"}"#,
            br#"{"name":"trip","title":"T","visibility":"public_listed"}"#,
            br#"{"name":"trip","title":"T","runtime":{"port":0}}"#,
            br#"{"name":"trip","title":"T","runtime":{"memory_mb":8192}}"#,
            br#"{"name":"trip","title":"T","runtime":{"privileged":true}}"#,
            br#"{"name":"trip","title":"T","runtime":{"health_path":"healthz"}}"#,
            br#"{"name":"trip","title":"T","runtime":{"env":{"PORT":"1"}}}"#,
            br#"{"name":"trip","title":"T","runtime":{"env":{"REPOBOX_APP":"1"}}}"#,
            br#"{"name":"trip","title":"T","runtime":{"env":{"A-B":"1"}}}"#,
            br#"{"name":"trip","title":"T","provenance":{"branch":"x"}}"#,
            br#"{"name":"trip","title":"T","ai":"yes"}"#,
            br#"[1]"#,
            br#"nope"#,
        ] {
            assert_eq!(
                parse(b).unwrap_err().code,
                "invalid_manifest",
                "{}",
                String::from_utf8_lossy(b)
            );
        }
        assert!(parse(&vec![b' '; MAX_MANIFEST_BYTES + 1]).is_err());
    }
}
