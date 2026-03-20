use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;

use crate::db;
use crate::git::{self, BackendRequest, RepoPath};
use crate::AppState;

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{address}/{repo}/info/refs", get(info_refs))
        .route("/{address}/{repo}/git-upload-pack", post(upload_pack))
        .route("/{address}/{repo}/git-receive-pack", post(receive_pack))
        .route("/{address}/{repo}/HEAD", get(head))
}

async fn info_refs(
    State(state): State<Arc<AppState>>,
    Path((address, repo)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    let service = match query.service.as_deref() {
        Some("git-upload-pack") | Some("git-receive-pack") => query.service.unwrap(),
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    if service == "git-receive-pack" {
        if let Err(error) = ensure_repo_initialized(&state, &repo) {
            return internal_error(error);
        }
    } else if !git::repo_dir(&state.data_dir, &repo).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let qs = if service == "git-upload-pack" {
        "service=git-upload-pack"
    } else {
        "service=git-receive-pack"
    };

    match git::run_backend(
        &state.data_dir,
        BackendRequest {
            method: "GET",
            path_info: format!("/{}/{}.git/info/refs", repo.address, repo.name),
            query_string: Some(qs),
            content_type: header_value(&headers, "content-type"),
            body: Bytes::new(),
        },
    ) {
        Ok(response) => response.into_response(),
        Err(error) => internal_error(error),
    }
}

async fn upload_pack(
    State(state): State<Arc<AppState>>,
    Path((address, repo)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    if !git::repo_dir(&state.data_dir, &repo).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    backend_post(
        &state,
        &repo,
        "/git-upload-pack",
        header_value(&headers, "content-type"),
        body,
    )
}

async fn receive_pack(
    State(state): State<Arc<AppState>>,
    Path((address, repo)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    let is_new_repo = !git::repo_dir(&state.data_dir, &repo).exists();

    if let Err(error) = ensure_repo_initialized(&state, &repo) {
        return internal_error(error);
    }

    let response = backend_post(
        &state,
        &repo,
        "/git-receive-pack",
        header_value(&headers, "content-type"),
        body,
    );

    // For new repos: require a signed first commit to establish ownership.
    // If unsigned or invalid, delete the repo — we only host compliant repos.
    if is_new_repo {
        tracing::debug!(
            repo = %format!("{}/{}", repo.address, repo.name),
            "new repo push completed, checking for EVM signature"
        );
        match git::extract_owner_from_first_commit(&state.data_dir, &repo) {
            Ok(Some(signer)) => {
                let _ = db::insert_repo_if_missing(&state.db_path, &repo.address, &repo.name, &signer);
                tracing::info!(
                    repo = %format!("{}/{}", repo.address, repo.name),
                    owner = %signer,
                    "ownership established via signed commit"
                );
            }
            _ => {
                // No valid signature — reject by removing the repo
                let repo_dir = git::repo_dir(&state.data_dir, &repo);
                let _ = std::fs::remove_dir_all(&repo_dir);
                tracing::warn!(
                    repo = %format!("{}/{}", repo.address, repo.name),
                    "rejected: first commit must be EVM-signed"
                );
                return (StatusCode::FORBIDDEN, "first commit must be EVM-signed to establish ownership").into_response();
            }
        }
    }

    response
}

async fn head(
    State(state): State<Arc<AppState>>,
    Path((address, repo)): Path<(String, String)>,
) -> Response {
    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    match git::read_head(&state.data_dir, &repo) {
        Ok(head) => {
            let mut response = head.into_response();
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            response
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
        Err(error) => internal_error(error),
    }
}

fn backend_post(
    state: &Arc<AppState>,
    repo: &RepoPath,
    suffix: &str,
    content_type: Option<&str>,
    body: Bytes,
) -> Response {
    match git::run_backend(
        &state.data_dir,
        BackendRequest {
            method: "POST",
            path_info: format!("/{}/{repo_name}.git{suffix}", repo.address, repo_name = repo.name),
            query_string: None,
            content_type,
            body,
        },
    ) {
        Ok(response) => response.into_response(),
        Err(error) => internal_error(error),
    }
}

fn repo_path(address: String, repo: String) -> Result<RepoPath, StatusCode> {
    git::validate_address(&address)?;
    let name = git::parse_repo(&repo)?;
    Ok(RepoPath { address, name })
}

fn ensure_repo_initialized(state: &AppState, repo: &RepoPath) -> std::io::Result<()> {
    // Just ensure the bare repo exists — ownership is established
    // after push via the EVM signature on the first commit.
    let _created = git::ensure_repo_exists(&state.data_dir, repo)?;
    Ok(())
}

fn header_value<'a>(headers: &'a HeaderMap, name: &'static str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn internal_error(error: std::io::Error) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
}

#[derive(Debug, serde::Deserialize)]
struct InfoRefsQuery {
    service: Option<String>,
}
