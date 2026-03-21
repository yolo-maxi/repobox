use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Serialize;

use crate::db;
use crate::git::{self, BackendRequest, RepoPath};
use crate::resolve;
use crate::AppState;

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Address-less routes (for auto-routing based on EVM signer)
        .route("/{repo}/info/refs", get(addressless_info_refs))
        .route("/{repo}/git-upload-pack", post(addressless_upload_pack))
        .route("/{repo}/git-receive-pack", post(addressless_receive_pack))
        .route("/{repo}/HEAD", get(addressless_head))
        // Regular two-segment routes (existing functionality)
        .route("/{address}/{repo}/info/refs", get(info_refs))
        .route("/{address}/{repo}/git-upload-pack", post(upload_pack))
        .route("/{address}/{repo}/git-receive-pack", post(receive_pack))
        .route("/{address}/{repo}/HEAD", get(head))
        .route("/{address}/{repo}/x402/grant-access", post(grant_access))
        // Name resolution route
        .route("/{name}/resolve", get(resolve_name))
}

async fn info_refs(
    State(state): State<Arc<AppState>>,
    Path((name, repo)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    // Resolve name to address
    let address = match resolve_name_to_address(&state, &name).await {
        Ok(addr) => addr,
        Err(status) => return status.into_response(),
    };

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

    // Check read access for clone/fetch operations
    if service == "git-upload-pack" {
        if let Err(denied) = check_read_access(&state, &repo, &headers) {
            return denied;
        }
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
    Path((name, repo)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Resolve name to address
    let address = match resolve_name_to_address(&state, &name).await {
        Ok(addr) => addr,
        Err(status) => return status.into_response(),
    };

    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    if !git::repo_dir(&state.data_dir, &repo).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Check read access
    if let Err(denied) = check_read_access(&state, &repo, &headers) {
        return denied;
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
    Path((name, repo)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Resolve name to address
    let address = match resolve_name_to_address(&state, &name).await {
        Ok(addr) => addr,
        Err(status) => return status.into_response(),
    };

    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

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

    // Log the push event after successful processing
    if response.status() == StatusCode::OK {
        if let Ok(Some(commit_info)) = git::extract_latest_commit_info(&state.data_dir, &repo) {
            let _ = db::insert_push_log(
                &state.db_path,
                &repo.address,
                &repo.name,
                commit_info.pusher_address.as_deref(),
                Some(&commit_info.hash),
                Some(&commit_info.message),
            );
            
            tracing::info!(
                repo = %format!("{}/{}", repo.address, repo.name),
                pusher = ?commit_info.pusher_address,
                commit = %commit_info.hash.get(..8).unwrap_or(&commit_info.hash),
                "push logged to activity feed"
            );
        }
    }

    // Check ownership and authorization.
    let existing_owner = db::get_repo(&state.db_path, &repo.address, &repo.name);

    if let Ok(Some(record)) = &existing_owner {
        // Check if repository has opted into permission enforcement
        let repo_dir = git::repo_dir(&state.data_dir, &repo);
        if read_config_from_repo(&repo_dir).is_some() {
            // Has config = permission enforcement enabled
            // Existing repo — check if the pusher is authorized.
            // Extract the signer of the HEAD commit (the latest pushed commit).
            if let Ok(Some(pusher)) = git::extract_pusher_from_head(&state.data_dir, &repo) {
                if !matches!(
                    git::check_push_authorized(&state.data_dir, &repo, &pusher, &record.owner_address),
                    Ok(true)
                ) {
                    // Unauthorized — revert would be ideal but for now just log.
                    // TODO: pre-receive hook integration for proper rejection.
                    tracing::warn!(
                        repo = %format!("{}/{}", repo.address, repo.name),
                        pusher = %pusher,
                        owner = %record.owner_address,
                        "push denied: pusher not authorized (config-enabled repo)"
                    );
                }
            }
        } else {
            // No config = no permission enforcement
            if let Ok(Some(pusher)) = git::extract_pusher_from_head(&state.data_dir, &repo) {
                tracing::debug!(
                    repo = %format!("{}/{}", repo.address, repo.name),
                    pusher = %pusher,
                    "push allowed: no .repobox/config.yml (permission enforcement disabled)"
                );
            }
        }
        // If HEAD commit is unsigned, the implicit owner-only rule still applies
        // through the pre-receive hook (when repobox-check is installed).
    } else {
        // New repo — first push. Verify the first commit is EVM-signed.
        tracing::debug!(
            repo = %format!("{}/{}", repo.address, repo.name),
            "checking first commit signature for new repo"
        );
        match git::extract_owner_from_first_commit(&state.data_dir, &repo) {
            Ok(Some(signer)) => {
                let _ = db::insert_repo_if_missing(&state.db_path, &repo.address, &repo.name, &signer);

                // Auto-assign alias if this is the first push for this address
                if db::get_alias_for_address(&state.db_path, &signer).unwrap_or(None).is_none() {
                    if let Ok(alias) = db::assign_random_alias(&state.db_path, &signer) {
                        tracing::info!(
                            address = %signer,
                            alias = %alias,
                            "auto-assigned alias for first push"
                        );
                    }
                }

                tracing::info!(
                    repo = %format!("{}/{}", repo.address, repo.name),
                    owner = %signer,
                    "ownership established via signed commit"
                );
            }
            _ => {
                // No valid signature — delete the repo
                // TEMP: Skip deletion for debugging
                let repo_dir = git::repo_dir(&state.data_dir, &repo);
                if let Err(e) = std::fs::remove_dir_all(&repo_dir) {
                    tracing::error!(
                        repo = %format!("{}/{}", repo.address, repo.name),
                        error = %e,
                        "failed to clean up unsigned repo"
                    );
                }
                tracing::warn!(
                    repo = %format!("{}/{}", repo.address, repo.name),
                    "rejected: first commit must be EVM-signed — repo deleted"
                );
            }
        }
    }

    response
}

async fn head(
    State(state): State<Arc<AppState>>,
    Path((name, repo)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    // Resolve name to address
    let address = match resolve_name_to_address(&state, &name).await {
        Ok(addr) => addr,
        Err(status) => return status.into_response(),
    };

    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    // Check read access
    if let Err(denied) = check_read_access(&state, &repo, &headers) {
        return denied;
    }

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

/// Resolve a name (address, ENS name, or alias) to an Ethereum address
async fn resolve_name_to_address(state: &AppState, name: &str) -> Result<String, StatusCode> {
    // Try to resolve as an alias first (before git validation)
    if let Ok(Some(address)) = db::resolve_alias(&state.db_path, name) {
        return Ok(address);
    }

    // Check if it's already a valid address format for the existing git validation
    // This allows compatibility with existing test addresses like "0xdemo"
    if git::validate_address(name).is_ok() {
        return Ok(name.to_string());
    }

    // Then try ENS resolution
    match resolve::resolve_ens_name(name).await {
        Ok(address) => Ok(address),
        Err(e) => {
            tracing::debug!("Failed to resolve name '{}': {}", name, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Check if the request has read access to the repo.
/// Returns Ok(()) if access is granted, Err(Response) if denied.
fn check_read_access(
    state: &AppState,
    repo: &RepoPath,
    headers: &HeaderMap,
) -> Result<(), Response> {
    let repo_dir = git::repo_dir(&state.data_dir, repo);

    // Try to read config from the repo (via git show HEAD:.repobox/config.yml)
    let config_content = match read_config_from_repo(&repo_dir) {
        Some(content) => content,
        None => {
            tracing::debug!(repo = %format!("{}/{}", repo.address, repo.name), 
                           "no .repobox/config.yml found - skipping permission enforcement");
            return Ok(()); // No config = no permission enforcement (public access)
        }
    };

    let config = match repobox::parser::parse(&config_content) {
        Ok(c) => c,
        Err(_) => return Ok(()), // Invalid config = don't block reads
    };

    // Check if any read rules exist
    let has_read_rules = config.permissions.rules.iter().any(|r| r.verb == repobox::config::Verb::Read);
    if !has_read_rules {
        // No read rules → use default policy
        return if config.permissions.default == repobox::config::DefaultPolicy::Allow {
            Ok(())
        } else {
            // default: deny with no read rules → need auth
            let repo_path_str = format!("{}/{}", repo.address, repo.name);
            match crate::auth::extract_identity(headers, &repo_path_str) {
                Ok(Some(_)) => {
                    // Authenticated but no read rules and default deny → denied
                    Err((StatusCode::FORBIDDEN, "read access denied").into_response())
                }
                Ok(None) => {
                    Err(unauthorized_response("authentication required"))
                }
                Err(e) => {
                    Err(unauthorized_response(&format!("auth error: {e}")))
                }
            }
        };
    }

    // Read rules exist — need to authenticate and check
    let repo_path_str = format!("{}/{}", repo.address, repo.name);
    let identity = match crate::auth::extract_identity(headers, &repo_path_str) {
        Ok(Some(id)) => id,
        Ok(None) => {
            return Err(unauthorized_response("authentication required for this repo"));
        }
        Err(e) => {
            return Err(unauthorized_response(&format!("auth error: {e}")));
        }
    };

    // Check read permission
    tracing::debug!(
        identity = %identity,
        config_groups = %config.groups.len(),
        config_rules = %config.permissions.rules.len(),
        "checking read permission"
    );

    // Debug the parsed rules
    for (i, rule) in config.permissions.rules.iter().enumerate() {
        tracing::debug!(rule_index = i, rule = ?rule, "parsed rule");
    }

    // Debug engine call parameters
    tracing::debug!(
        engine_identity = ?identity,
        engine_verb = ?repobox::config::Verb::Read,
        engine_branch = ?Option::<&str>::None,
        engine_file = ?Option::<&str>::None,
        "calling engine::check"
    );

    let result = repobox::engine::check(&config, &identity, repobox::config::Verb::Read, None, None);
    if result.is_allowed() {
        tracing::debug!(identity = %identity, "read access granted");
        Ok(())
    } else {
        tracing::warn!(identity = %identity, result = ?result, "read access denied");

        // Check if x402 payment is configured
        if let Some(x402_config) = &config.x402 {
            // Return 402 Payment Required with x402 payment headers
            let payment_json = serde_json::json!({
                "scheme": "exact",
                "network": x402_config.network,
                "currency": "USDC",
                "amount": parse_usdc_amount(&x402_config.read_price).unwrap_or_else(|| "1000000".to_string()),
                "recipient": x402_config.recipient,
                "memo": format!("read:{}", repo.name)
            });

            let mut response = (StatusCode::PAYMENT_REQUIRED, "payment required for read access").into_response();
            response.headers_mut().insert(
                "X-Payment",
                HeaderValue::from_str(&payment_json.to_string()).unwrap(),
            );
            tracing::info!(
                identity = %identity,
                repo = %format!("{}/{}", repo.address, repo.name),
                price = %x402_config.read_price,
                "returning 402 payment required"
            );
            Err(response)
        } else {
            Err((StatusCode::FORBIDDEN, format!("read access denied for {}", identity)).into_response())
        }
    }
}

/// Read .repobox/config.yml from a bare git repo.
/// Uses the first available branch (not HEAD, which may point to a non-existent branch).
pub(crate) fn read_config_from_repo(repo_dir: &std::path::Path) -> Option<String> {
    tracing::debug!(repo_dir = ?repo_dir, "reading config from repo");
    
    // First try HEAD
    let output = std::process::Command::new("git")
        .args(["show", "HEAD:.repobox/config.yml"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if output.status.success() {
        let config = String::from_utf8_lossy(&output.stdout).to_string();
        tracing::debug!(config_len = config.len(), "read config via HEAD");
        return Some(config);
    }
    
    tracing::debug!("HEAD failed, trying first branch fallback");

    // HEAD failed (probably points to non-existent default branch) — find first available ref
    let refs_output = std::process::Command::new("git")
        .args(["for-each-ref", "--format=%(refname:short)", "refs/heads"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    let first_branch = String::from_utf8_lossy(&refs_output.stdout)
        .lines()
        .next()?
        .to_string();
    
    tracing::debug!(first_branch = %first_branch, "trying config from first branch");

    let output = std::process::Command::new("git")
        .args(["show", &format!("{}:.repobox/config.yml", first_branch)])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if output.status.success() {
        let config = String::from_utf8_lossy(&output.stdout).to_string();
        tracing::debug!(config_len = config.len(), "read config via fallback");
        Some(config)
    } else {
        tracing::warn!("fallback config read failed");
        None
    }
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

/// Return a 401 response with WWW-Authenticate header so git invokes its credential helper.
fn unauthorized_response(message: &str) -> Response {
    let mut response = (StatusCode::UNAUTHORIZED, message.to_string()).into_response();
    response.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"repo.box\""),
    );
    response
}

#[derive(Debug, serde::Deserialize)]
struct InfoRefsQuery {
    service: Option<String>,
}

// Address-less route handlers
async fn addressless_info_refs(
    State(state): State<Arc<AppState>>,
    Path(repo): Path<String>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    let repo_name = match git::parse_repo(&repo) {
        Ok(name) => name,
        Err(status) => return status.into_response(),
    };

    let _service = match query.service.as_deref() {
        Some("git-upload-pack") => {
            // For clones/fetches, require the full /{address}/{repo}.git path
            return StatusCode::NOT_FOUND.into_response();
        }
        Some("git-receive-pack") => query.service.unwrap(),
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    // Check if this repo already exists under some address
    if let Ok(Some(existing)) = db::find_repo_by_name(&state.db_path, &repo_name) {
        // Repo exists - proxy to the existing location
        let repo_path = RepoPath {
            address: existing.address,
            name: repo_name,
        };
        let qs = "service=git-receive-pack";
        
        match git::run_backend(
            &state.data_dir,
            BackendRequest {
                method: "GET",
                path_info: format!("/{}/{}.git/info/refs", repo_path.address, repo_path.name),
                query_string: Some(qs),
                content_type: header_value(&headers, "content-type"),
                body: Bytes::new(),
            },
        ) {
            Ok(response) => response.into_response(),
            Err(error) => internal_error(error),
        }
    } else {
        // New repo - create staging repo
        if let Err(error) = git::ensure_staging_repo_exists(&state.data_dir, &repo_name) {
            return internal_error(error);
        }

        let _staging_dir = git::staging_repo_dir(&state.data_dir, &repo_name);
        let qs = "service=git-receive-pack";

        match git::run_backend(
            &state.data_dir,
            BackendRequest {
                method: "GET",
                path_info: format!("/_staging/{}.git/info/refs", repo_name),
                query_string: Some(qs),
                content_type: header_value(&headers, "content-type"),
                body: Bytes::new(),
            },
        ) {
            Ok(response) => response.into_response(),
            Err(error) => internal_error(error),
        }
    }
}

async fn addressless_upload_pack(
    State(_state): State<Arc<AppState>>,
    Path(_repo): Path<String>,
    _headers: HeaderMap,
    _body: Bytes,
) -> Response {
    // Clones/fetches require the full /{address}/{repo}.git path
    StatusCode::NOT_FOUND.into_response()
}

async fn addressless_receive_pack(
    State(state): State<Arc<AppState>>,
    Path(repo): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo_name = match git::parse_repo(&repo) {
        Ok(name) => name,
        Err(status) => return status.into_response(),
    };

    // Check if this repo already exists under some address
    if let Ok(Some(existing)) = db::find_repo_by_name(&state.db_path, &repo_name) {
        // Repo exists - proxy to the existing location
        let repo_path = RepoPath {
            address: existing.address,
            name: repo_name,
        };
        
        let response = backend_post(
            &state,
            &repo_path,
            "/git-receive-pack",
            header_value(&headers, "content-type"),
            body,
        );

        // Check ownership and authorization for addressless pushes to existing repos
        if response.status() == StatusCode::OK {
            let existing_owner = db::get_repo(&state.db_path, &repo_path.address, &repo_path.name);
            
            if let Ok(Some(record)) = &existing_owner {
                // Check if repository has opted into permission enforcement
                let repo_dir = git::repo_dir(&state.data_dir, &repo_path);
                if read_config_from_repo(&repo_dir).is_some() {
                    // Has config = permission enforcement enabled
                    if let Ok(Some(pusher)) = git::extract_pusher_from_head(&state.data_dir, &repo_path) {
                        if !matches!(
                            git::check_push_authorized(&state.data_dir, &repo_path, &pusher, &record.owner_address),
                            Ok(true)
                        ) {
                            tracing::warn!(
                                repo = %format!("{}/{}", repo_path.address, repo_path.name),
                                pusher = %pusher,
                                owner = %record.owner_address,
                                "addressless push denied: pusher not authorized (config-enabled repo)"
                            );
                        }
                    }
                } else {
                    // No config = no permission enforcement
                    if let Ok(Some(pusher)) = git::extract_pusher_from_head(&state.data_dir, &repo_path) {
                        tracing::debug!(
                            repo = %format!("{}/{}", repo_path.address, repo_path.name),
                            pusher = %pusher,
                            "addressless push allowed: no .repobox/config.yml (permission enforcement disabled)"
                        );
                    }
                }
            }
            
            // Log the push event for existing repos accessed via addressless route
            if let Ok(Some(commit_info)) = git::extract_latest_commit_info(&state.data_dir, &repo_path) {
                let _ = db::insert_push_log(
                    &state.db_path,
                    &repo_path.address,
                    &repo_path.name,
                    commit_info.pusher_address.as_deref(),
                    Some(&commit_info.hash),
                    Some(&commit_info.message),
                );
                
                tracing::info!(
                    repo = %format!("{}/{}", repo_path.address, repo_path.name),
                    pusher = ?commit_info.pusher_address,
                    commit = %commit_info.hash.get(..8).unwrap_or(&commit_info.hash),
                    "addressless push to existing repo logged to activity feed"
                );
            }
        }

        response
    } else {
        // New repo - use staging area
        if let Err(error) = git::ensure_staging_repo_exists(&state.data_dir, &repo_name) {
            return internal_error(error);
        }

        // Process the push to staging
        let response = match git::run_backend(
            &state.data_dir,
            BackendRequest {
                method: "POST",
                path_info: format!("/_staging/{}.git/git-receive-pack", repo_name),
                query_string: None,
                content_type: header_value(&headers, "content-type"),
                body,
            },
        ) {
            Ok(response) => response,
            Err(error) => return internal_error(error),
        };

        // Extract the signer from the first commit
        match git::extract_owner_from_staging_repo(&state.data_dir, &repo_name) {
            Ok(Some(signer)) => {
                // Move from staging to final location
                if let Err(error) = git::move_repo_from_staging(&state.data_dir, &repo_name, &signer) {
                    let _ = git::clean_staging_repo(&state.data_dir, &repo_name);
                    return internal_error(error);
                }
                
                // Record ownership in database
                let _ = db::insert_repo_if_missing(&state.db_path, &signer, &repo_name, &signer);

                // Auto-assign alias if this is the first push for this address
                if db::get_alias_for_address(&state.db_path, &signer).unwrap_or(None).is_none() {
                    if let Ok(alias) = db::assign_random_alias(&state.db_path, &signer) {
                        tracing::info!(
                            address = %signer,
                            alias = %alias,
                            "auto-assigned alias for first address-less push"
                        );
                    }
                }

                tracing::info!(
                    repo = %format!("{}/{}", signer, repo_name),
                    owner = %signer,
                    "ownership established via address-less push"
                );

                // Log the push event for addressless pushes too
                let repo_path = RepoPath {
                    address: signer.clone(),
                    name: repo_name.clone(),
                };
                if response.status() == StatusCode::OK {
                    if let Ok(Some(commit_info)) = git::extract_latest_commit_info(&state.data_dir, &repo_path) {
                        let _ = db::insert_push_log(
                            &state.db_path,
                            &signer,
                            &repo_name,
                            commit_info.pusher_address.as_deref(),
                            Some(&commit_info.hash),
                            Some(&commit_info.message),
                        );
                        
                        tracing::info!(
                            repo = %format!("{}/{}", signer, repo_name),
                            pusher = ?commit_info.pusher_address,
                            commit = %commit_info.hash.get(..8).unwrap_or(&commit_info.hash),
                            "addressless push logged to activity feed"
                        );
                    }
                }

                response.into_response()
            }
            _ => {
                // No valid signature — clean up staging repo
                let _ = git::clean_staging_repo(&state.data_dir, &repo_name);
                tracing::warn!(
                    repo = %repo_name,
                    "rejected: address-less push requires EVM-signed first commit"
                );
                response.into_response()
            }
        }
    }
}

async fn addressless_head(
    State(state): State<Arc<AppState>>,
    Path(repo): Path<String>,
) -> Response {
    let repo_name = match git::parse_repo(&repo) {
        Ok(name) => name,
        Err(status) => return status.into_response(),
    };

    // Check if this repo exists under some address
    if let Ok(Some(existing)) = db::find_repo_by_name(&state.db_path, &repo_name) {
        let repo_path = RepoPath {
            address: existing.address,
            name: repo_name,
        };
        match git::read_head(&state.data_dir, &repo_path) {
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
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn grant_access(
    State(state): State<Arc<AppState>>,
    Path((name, repo)): Path<(String, String)>,
    axum::extract::Json(payload): axum::extract::Json<GrantAccessRequest>,
) -> Response {
    // Resolve name to address
    let address = match resolve_name_to_address(&state, &name).await {
        Ok(addr) => addr,
        Err(status) => return status.into_response(),
    };

    let repo = match repo_path(address, repo) {
        Ok(repo) => repo,
        Err(status) => return status.into_response(),
    };

    // Read config to verify x402 is enabled
    let repo_dir = git::repo_dir(&state.data_dir, &repo);
    let config_content = match read_config_from_repo(&repo_dir) {
        Some(content) => content,
        None => return (StatusCode::NOT_FOUND, "no x402 config found").into_response(),
    };

    let mut config = match repobox::parser::parse(&config_content) {
        Ok(c) => c,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid config").into_response(),
    };

    let _x402_config = match &config.x402 {
        Some(cfg) => cfg,
        None => return (StatusCode::NOT_FOUND, "x402 not enabled for this repo").into_response(),
    };

    // TODO: Verify payment on-chain using payload.tx_hash
    // For MVP, we'll skip verification and just grant access

    tracing::info!(
        repo = %format!("{}/{}", repo.address, repo.name),
        payer_address = %payload.address,
        tx_hash = %payload.tx_hash,
        "granting paid read access (payment verification skipped for MVP)"
    );

    // Add the address to the paid-readers group
    let paid_readers_group = config.groups.entry("paid-readers".to_string()).or_insert_with(|| {
        repobox::config::Group {
            name: "paid-readers".to_string(),
            members: vec![],
            includes: vec![],
            resolver: None,
        }
    });

    let identity = match repobox::config::Identity::parse(&format!("evm:{}", payload.address)) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid address format").into_response(),
    };

    if !paid_readers_group.members.contains(&identity) {
        paid_readers_group.members.push(identity);

        // Write updated config back to the repo
        if let Err(e) = write_config_to_repo(&repo_dir, &config) {
            tracing::error!(error = %e, "failed to update config");
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to update config").into_response();
        }
    }

    (StatusCode::OK, "access granted").into_response()
}

#[derive(serde::Deserialize)]
struct GrantAccessRequest {
    address: String,
    tx_hash: String,
}

/// Convert USDC amount string (e.g., "1.00") to raw amount (e.g., "1000000").
/// USDC has 6 decimal places.
fn parse_usdc_amount(price_str: &str) -> Option<String> {
    let amount: f64 = price_str.parse().ok()?;
    let raw_amount = (amount * 1_000_000.0) as u64;
    Some(raw_amount.to_string())
}

/// Write updated config back to the repository.
/// For MVP, this updates the config in the current HEAD branch.
fn write_config_to_repo(repo_dir: &std::path::Path, config: &repobox::config::Config) -> Result<(), std::io::Error> {
    // For MVP, we'll create a simple YAML representation
    // In production, you'd want to preserve the original YAML structure/comments
    let mut yaml_content = String::new();

    // Write groups
    if !config.groups.is_empty() {
        yaml_content.push_str("groups:\n");
        for (name, group) in &config.groups {
            yaml_content.push_str(&format!("  {}:\n", name));
            if !group.members.is_empty() {
                yaml_content.push_str("    members:\n");
                for member in &group.members {
                    yaml_content.push_str(&format!("      - {}\n", member));
                }
            }
            if !group.includes.is_empty() {
                yaml_content.push_str("    includes:\n");
                for include in &group.includes {
                    yaml_content.push_str(&format!("      - {}\n", include));
                }
            }
        }
        yaml_content.push('\n');
    }

    // Write permissions
    yaml_content.push_str("permissions:\n");
    let default_str = match config.permissions.default {
        repobox::config::DefaultPolicy::Allow => "allow",
        repobox::config::DefaultPolicy::Deny => "deny",
    };
    yaml_content.push_str(&format!("  default: {}\n", default_str));
    yaml_content.push_str("  rules:\n");
    for rule in &config.permissions.rules {
        let subject_str = match &rule.subject {
            repobox::config::Subject::All => "*".to_string(),
            repobox::config::Subject::Group(name) => name.clone(),
            repobox::config::Subject::Identity(id) => id.to_string(),
        };
        let deny_str = if rule.deny { "not " } else { "" };
        let target_str = if let Some(branch) = &rule.target.branch {
            if let Some(path) = &rule.target.path {
                format!("{} >{}", path, branch)
            } else {
                format!(">{}", branch)
            }
        } else if let Some(path) = &rule.target.path {
            path.clone()
        } else {
            "*".to_string()
        };
        yaml_content.push_str(&format!("    - \"{} {}{} {}\"\n", subject_str, deny_str, rule.verb, target_str));
    }

    // Write x402 config if present
    if let Some(x402) = &config.x402 {
        yaml_content.push_str("\nx402:\n");
        yaml_content.push_str(&format!("  read_price: \"{}\"\n", x402.read_price));
        yaml_content.push_str(&format!("  recipient: \"{}\"\n", x402.recipient));
        yaml_content.push_str(&format!("  network: \"{}\"\n", x402.network));
    }

    // Write to a temporary file and then commit it
    let temp_file = std::env::temp_dir().join(format!("config-{}.yml", std::process::id()));
    std::fs::write(&temp_file, yaml_content)?;

    // Use git to update the config in the repo
    let output = std::process::Command::new("git")
        .args(["hash-object", "-w"])
        .arg(&temp_file)
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to hash config object"
        ));
    }

    let config_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Update the index
    let output = std::process::Command::new("git")
        .args(["update-index", "--add", "--cacheinfo", "100644", &config_hash, ".repobox/config.yml"])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to update index"
        ));
    }

    // Create a new commit
    let tree_output = std::process::Command::new("git")
        .args(["write-tree"])
        .current_dir(repo_dir)
        .output()?;

    if !tree_output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to write tree"
        ));
    }

    let tree_hash = String::from_utf8_lossy(&tree_output.stdout).trim().to_string();

    // Get current HEAD
    let head_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()?;

    let parent_arg = if head_output.status.success() {
        let head_hash = String::from_utf8_lossy(&head_output.stdout).trim().to_string();
        vec!["-p".to_string(), head_hash]
    } else {
        vec![]
    };

    // Create commit
    let mut commit_args = vec!["commit-tree".to_string(), tree_hash, "-m".to_string(), "x402: grant paid read access".to_string()];
    commit_args.extend(parent_arg);

    let commit_output = std::process::Command::new("git")
        .args(&commit_args)
        .current_dir(repo_dir)
        .output()?;

    if !commit_output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to create commit"
        ));
    }

    let commit_hash = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();

    // Update HEAD
    std::process::Command::new("git")
        .args(["update-ref", "HEAD", &commit_hash])
        .current_dir(repo_dir)
        .output()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(())
}

async fn resolve_name(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Response {
    #[derive(Serialize)]
    struct ResolveResponse {
        address: String,
        source: String,
    }

    let (address, source) = if name.starts_with("0x") && name.len() == 42 && name[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        // Direct address
        (name.to_lowercase(), "direct".to_string())
    } else if let Ok(Some(addr)) = db::resolve_alias(&state.db_path, &name) {
        // Alias resolution
        (addr, "alias".to_string())
    } else {
        // Try ENS resolution
        match resolve::resolve_ens_name(&name).await {
            Ok(addr) => (addr, "ens".to_string()),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to resolve name '{}': {}", name, e)
                ).into_response();
            }
        }
    };

    axum::Json(ResolveResponse { address, source }).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempdir::TempDir;

    fn create_test_state() -> Arc<AppState> {
        let temp_dir = TempDir::new("repobox_test").unwrap();
        let data_dir = temp_dir.into_path();
        Arc::new(AppState::new(data_dir, None).unwrap())
    }

    #[tokio::test]
    async fn test_resolve_name_to_address() {
        let state = create_test_state();

        // Test with a format that passes git validation (like test addresses in existing tests)
        let address = "0xdemo";
        let result = resolve_name_to_address(&state, address).await.unwrap();
        assert_eq!(result, address);

        // Test alias resolution - use a different address to avoid conflicts
        let real_address = "0xabcdef1234567890abcdef1234567890abcdef12";
        let alias = db::assign_random_alias(&state.db_path, real_address).unwrap();

        // Verify the alias was stored correctly
        let resolved_from_db = db::resolve_alias(&state.db_path, &alias).unwrap();
        assert_eq!(resolved_from_db, Some(real_address.to_string()));

        let result = resolve_name_to_address(&state, &alias).await.unwrap();
        assert_eq!(result, real_address);
    }
}
