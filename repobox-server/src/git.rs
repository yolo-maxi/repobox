use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use axum::body::{Body, Bytes};
use http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepoPath {
    pub address: String,
    pub name: String,
}

#[derive(Debug)]
pub(crate) struct BackendRequest<'a> {
    pub method: &'a str,
    pub path_info: String,
    pub query_string: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub body: Bytes,
}

pub(crate) fn parse_repo(repo: &str) -> Result<String, StatusCode> {
    let Some(name) = repo.strip_suffix(".git") else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !is_safe_segment(name) || name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(name.to_string())
}

pub(crate) fn validate_address(address: &str) -> Result<(), StatusCode> {
    if is_safe_segment(address) && !address.is_empty() {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub(crate) fn repo_dir(data_dir: &Path, repo: &RepoPath) -> PathBuf {
    data_dir.join(&repo.address).join(format!("{}.git", repo.name))
}

pub(crate) fn ensure_repo_exists(data_dir: &Path, repo: &RepoPath) -> std::io::Result<bool> {
    let repo_dir = repo_dir(data_dir, repo);
    if repo_dir.exists() {
        return Ok(false);
    }

    let parent = repo_dir
        .parent()
        .ok_or_else(|| std::io::Error::other("invalid repository path"))?;
    std::fs::create_dir_all(parent)?;

    run_git(["init", "--bare", repo_dir.to_string_lossy().as_ref()])?;
    run_git([
        "--git-dir",
        repo_dir.to_string_lossy().as_ref(),
        "config",
        "http.receivepack",
        "true",
    ])?;

    install_pre_receive_hook(&repo_dir)?;
    Ok(true)
}

pub(crate) fn read_head(data_dir: &Path, repo: &RepoPath) -> std::io::Result<String> {
    std::fs::read_to_string(repo_dir(data_dir, repo).join("HEAD"))
}

pub(crate) fn run_backend(
    data_dir: &Path,
    request: BackendRequest<'_>,
) -> std::io::Result<Response<Body>> {
    let mut command = Command::new("git");
    command.arg("http-backend");
    command.env("GIT_PROJECT_ROOT", data_dir);
    command.env("GIT_HTTP_EXPORT_ALL", "1");
    command.env("REQUEST_METHOD", request.method);
    command.env("PATH_INFO", &request.path_info);
    command.env("QUERY_STRING", request.query_string.unwrap_or_default());
    command.env("CONTENT_TYPE", request.content_type.unwrap_or_default());
    command.env("CONTENT_LENGTH", request.body.len().to_string());
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(&request.body)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "git http-backend failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    parse_backend_response(&output.stdout)
}

fn parse_backend_response(output: &[u8]) -> std::io::Result<Response<Body>> {
    let (raw_headers, body) = split_cgi_response(output)?;
    let headers_text = String::from_utf8_lossy(raw_headers);
    let mut status = StatusCode::OK;
    let mut headers = HeaderMap::new();

    for line in headers_text.lines() {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() {
            continue;
        }

        let Some((name, value)) = trimmed.split_once(':') else {
            continue;
        };

        if name.eq_ignore_ascii_case("Status") {
            let code = value
                .trim()
                .split_whitespace()
                .next()
                .and_then(|part| part.parse::<u16>().ok())
                .ok_or_else(|| std::io::Error::other("invalid CGI status line"))?;
            status = StatusCode::from_u16(code)
                .map_err(|_| std::io::Error::other("invalid HTTP status code"))?;
            continue;
        }

        let header_name = HeaderName::from_bytes(name.trim().as_bytes())
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let header_value = HeaderValue::from_str(value.trim())
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        headers.append(header_name, header_value);
    }

    let mut builder = Response::builder().status(status);
    let response_headers = builder
        .headers_mut()
        .ok_or_else(|| std::io::Error::other("failed to build response headers"))?;
    response_headers.extend(headers);
    builder
        .body(Body::from(body.to_vec()))
        .map_err(|error| std::io::Error::other(error.to_string()))
}

fn split_cgi_response(output: &[u8]) -> std::io::Result<(&[u8], &[u8])> {
    if let Some(index) = output.windows(4).position(|window| window == b"\r\n\r\n") {
        return Ok((&output[..index], &output[index + 4..]));
    }
    if let Some(index) = output.windows(2).position(|window| window == b"\n\n") {
        return Ok((&output[..index], &output[index + 2..]));
    }
    Err(std::io::Error::other("malformed CGI response"))
}

fn install_pre_receive_hook(repo_dir: &Path) -> std::io::Result<()> {
    let hook_path = repo_dir.join("hooks").join("pre-receive");
    let script = "#!/bin/sh\nif command -v repobox-check >/dev/null 2>&1; then\n    exec repobox-check \"$@\"\nfi\nexit 0\n";
    std::fs::write(&hook_path, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, permissions)?;
    }

    Ok(())
}

fn run_git<const N: usize>(args: [&str; N]) -> std::io::Result<()> {
    let output = Command::new("git").args(args).output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(std::io::Error::other(format!(
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    )))
}

fn is_safe_segment(segment: &str) -> bool {
    !segment.contains("..")
        && !segment.contains('/')
        && !segment.contains('\\')
        && !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b':' ))
}
