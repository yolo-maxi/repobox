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
        .route("/:address/:repo/info/refs", get(info_refs))
        .route("/:address/:repo/git-upload-pack", post(upload_pack))
        .route("/:address/:repo/git-receive-pack", post(receive_pack))
        .route("/:address/:repo/HEAD", get(head))
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

    match git::run_backend(
        &state.data_dir,
        BackendRequest {
            method: "GET",
            path_info: format!("/{}/{repo_name}.git/info/refs", repo.address, repo_name = repo.name),
            query_string: query.service.as_deref().map(|_| service.as_str()).map(|_| {
                if service == "git-upload-pack" {
                    "service=git-upload-pack"
                } else {
                    "service=git-receive-pack"
                }
            }),
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

    if let Err(error) = ensure_repo_initialized(&state, &repo) {
        return internal_error(error);
    }

    backend_post(
        &state,
        &repo,
        "/git-receive-pack",
        header_value(&headers, "content-type"),
        body,
    )
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
    let created = git::ensure_repo_exists(&state.data_dir, repo)?;
    if created {
        db::insert_repo_if_missing(&state.db_path, &repo.address, &repo.name, &repo.address)?;
    }
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
