//! Agent/operator discovery and the scoped machine API.
//!
//! * `GET /api/platform/v1` (also `/.well-known/repobox-platform.json`):
//!   public, versioned capabilities document. No secrets, no registry data.
//! * `GET /api/platform/v1/openapi.json`, `GET /api/platform/v1/skill.md`:
//!   the machine-readable contract and the agent skill.
//! * Everything else needs `Authorization: Bearer rbp_…`, a **service
//!   token**: operator-issued with the CLI (`service-token create`), bound to
//!   one owner, scoped, optionally narrowed to named apps, expiring,
//!   revocable, hashed at rest. It is *not* OAuth; an OAuth client-credentials
//!   issuer is the documented remaining dependency.
//! * `POST /api/platform/v1/mcp`: the same operations as MCP tools
//!   (JSON-RPC 2.0 over streamable HTTP, JSON responses only), same bearer.
//!
//! A token only ever sees apps its owner owns. There is no operation for
//! users, grants, sessions, Caddy apply, SSH, the database or provider
//! credentials, and app registration is a *request* an operator approves
//! with the CLI (the canonical mutating interface).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use super::S;
use crate::model::{
    AI_APP_DAILY_CEILING, AI_CHAT_PATH, AI_KNOWN_MODELS, AI_MAX_BODY_BYTES,
    AI_MAX_INPUT_CHARS_CEILING, AI_MAX_MESSAGES, AI_MAX_OUTPUT_TOKENS_CEILING, AI_MODELS_PATH,
    AI_PUBLIC_POLICY_SIGNED_IN_QUOTA, AI_USER_DAILY_CEILING, AiPolicy, App, AppKind,
    SERVICE_SCOPES, User, Visibility,
};
use crate::store::ServiceToken;

pub const API_VERSION: &str = "v1";
pub const SKILL_MD: &str = include_str!("../../../skills/repobox-platform/SKILL.md");
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

fn build_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn build_commit() -> &'static str {
    option_env!("REPOBOX_PLATFORM_GIT_SHA").unwrap_or("unknown")
}

// ----------------------------------------------------------------- errors

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    fn unauthorized(s: &S) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            format!(
                "a valid service token is required (Authorization: Bearer rbp_…); docs: {}",
                super::docs::docs_url(&s.cfg.public_base)
            ),
        )
    }
    fn scope(scope: &str) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "insufficient_scope",
            format!("this token lacks the '{scope}' scope"),
        )
    }
    fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "no such app for this token",
        )
    }
    fn invalid(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid", message)
    }
    fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "internal error",
        )
    }
    fn body(&self) -> Value {
        json!({ "error": { "code": self.code, "message": self.message } })
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut r = json_ok_status(self.status, &self.body());
        if self.status == StatusCode::UNAUTHORIZED {
            r.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"repo.box platform\""),
            );
        }
        r
    }
}

fn json_ok_status(status: StatusCode, v: &Value) -> Response {
    let mut r = (status, v.to_string()).into_response();
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

fn reply(r: Result<Value, ApiError>) -> Response {
    match r {
        Ok(v) => json_ok_status(StatusCode::OK, &v),
        Err(e) => e.into_response(),
    }
}

// ------------------------------------------------------------------- auth

pub struct Caller {
    pub token: ServiceToken,
    pub owner: User,
}

fn caller(s: &S, headers: &HeaderMap) -> Result<Caller, ApiError> {
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::unauthorized(s))?;
    match s.store.service_token_lookup(raw.trim()) {
        Ok(Some((token, owner))) => Ok(Caller { token, owner }),
        Ok(None) => Err(ApiError::unauthorized(s)),
        Err(_) => Err(ApiError::internal()),
    }
}

impl Caller {
    fn need(&self, scope: &str) -> Result<(), ApiError> {
        if self.token.has_scope(scope) {
            Ok(())
        } else {
            Err(ApiError::scope(scope))
        }
    }

    fn app(&self, s: &S, name: &str) -> Result<App, ApiError> {
        let app = s
            .store
            .app_by_name(name)
            .map_err(|_| ApiError::internal())?
            .ok_or_else(ApiError::not_found)?;
        if !self.token.reaches(&app) {
            return Err(ApiError::not_found());
        }
        Ok(app)
    }

    fn audit(&self, s: &S, action: &str, subject: &str, detail: &str) {
        s.store.audit(
            Some(self.owner.id),
            action,
            subject,
            &format!("service-token={} {detail}", self.token.name),
        );
    }
}

// -------------------------------------------------------------- documents

pub fn discovery_doc(s: &S) -> Value {
    let base = &s.cfg.public_base;
    json!({
        "service": "repo.box platform control plane",
        "api_version": API_VERSION,
        "software_version": build_version(),
        "docs": super::docs::docs_url(base),
        "documentation": {
            "docs": super::docs::docs_url(base),
            "publisher_quickstart": format!("{}#publish", super::docs::docs_url(base)),
            "skill": format!("{base}/api/platform/v1/skill.md"),
            "openapi": format!("{base}/api/platform/v1/openapi.json"),
            "cli": "repobox-platform --help (operator host); every subcommand has --help",
        },
        "authentication": {
            "type": "service_token",
            "scheme": "Authorization: Bearer rbp_<43 base64url chars>",
            "issued_by": "an operator with `repobox-platform service-token create` (written once to a 0600 file)",
            "binding": "one owner; only that owner's apps, optionally narrowed to named apps; expires (<=90 days); revocable",
            "oauth": "not implemented; a scoped OAuth client-credentials issuer is the remaining dependency. Do not treat this as OAuth.",
            "scopes": SERVICE_SCOPES.iter().map(|(n, d)| json!({"name": n, "description": d})).collect::<Vec<_>>(),
            "never_granted": ["SSH", "Caddy apply/reload", "database access", "user/grant/session administration", "provider (ChatMock/ChatGPT) credentials"],
        },
        "endpoints": {
            "discovery": format!("{base}/api/platform/v1"),
            "docs": format!("GET {} (HTML, public)", super::docs::docs_url(base)),
            "whoami": format!("GET {base}/api/platform/v1/whoami"),
            "apps": format!("GET {base}/api/platform/v1/apps"),
            "app": format!("GET {base}/api/platform/v1/apps/{{name}}"),
            "ai_policy": format!("GET|PATCH {base}/api/platform/v1/apps/{{name}}/ai"),
            "route_preview": format!("GET {base}/api/platform/v1/apps/{{name}}/route"),
            "app_requests": format!("GET|POST {base}/api/platform/v1/app-requests"),
            "release": format!("GET {base}/api/platform/v1/release"),
            "mcp": format!("POST {base}/api/platform/v1/mcp"),
        },
        "mcp": {
            "endpoint": format!("{base}/api/platform/v1/mcp"),
            "transport": "streamable HTTP (JSON responses only, no SSE)",
            "protocol_version": MCP_PROTOCOL_VERSION,
            "auth": "same bearer service token",
            "tools": mcp_tools().iter().map(|t| t["name"].clone()).collect::<Vec<_>>(),
        },
        "app_ai_endpoint": {
            "url": format!("https://<app>.{}{AI_CHAT_PATH}", s.cfg.domain),
            "models_url": format!("https://<app>.{}{AI_MODELS_PATH}", s.cfg.domain),
            "compatibility": "OpenAI chat.completions (subset)",
            "auth": "same-origin browser request carrying the app's platform session (launch from auth.repo.box); no API key, no per-app credential",
            "streaming": false,
            "tools": false,
            "provider": "chatmock (via the platform broker; never exposed)",
            "models": AI_KNOWN_MODELS,
            "limits": {
                "max_body_bytes": AI_MAX_BODY_BYTES,
                "max_messages": AI_MAX_MESSAGES,
                "max_input_chars_ceiling": AI_MAX_INPUT_CHARS_CEILING,
                "max_output_tokens_ceiling": AI_MAX_OUTPUT_TOKENS_CEILING,
                "user_daily_requests_ceiling": AI_USER_DAILY_CEILING,
                "app_daily_requests_ceiling": AI_APP_DAILY_CEILING,
            },
            "defaults": AiPolicy::private_default().to_json(),
            "public_apps": format!("AI stays off unless the app declares public_policy '{AI_PUBLIC_POLICY_SIGNED_IN_QUOTA}' with explicit quotas; even then only signed-in platform users with access can call it"),
        },
        "publishing": super::publisher::discovery(s),
    })
}

pub async fn discovery(State(s): State<S>) -> Response {
    let mut r = json_ok_status(StatusCode::OK, &discovery_doc(&s));
    r.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );
    r
}

pub async fn skill() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/markdown; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
        ],
        SKILL_MD,
    )
        .into_response()
}

pub async fn openapi(State(s): State<S>) -> Response {
    let base = &s.cfg.public_base;
    let err = json!({"$ref": "#/components/schemas/Error"});
    let std_errors = json!({
        "401": {"description": "missing or invalid service token", "content": {"application/json": {"schema": err}}},
        "403": {"description": "token lacks the scope", "content": {"application/json": {"schema": err}}},
        "404": {"description": "no such app for this token", "content": {"application/json": {"schema": err}}},
    });
    let name =
        json!([{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}]);
    let op = |summary: &str, scope: &str, extra: Value| {
        let mut o = json!({
            "summary": summary,
            "security": [{"serviceToken": [scope]}],
            "responses": {"200": {"description": "OK", "content": {"application/json": {}}}},
        });
        for (k, v) in std_errors.as_object().unwrap() {
            o["responses"][k] = v.clone();
        }
        if let Value::Object(m) = extra {
            for (k, v) in m {
                o[k] = v;
            }
        }
        o
    };
    let doc = json!({
        "openapi": "3.1.0",
        "info": {
            "title": "repo.box platform control plane API",
            "version": API_VERSION,
            "description": "Scoped machine API for agents. Service tokens (rbp_) expose owner-scoped reads, bounded AI policy updates and app registration requests. Publisher tokens (rbpub_) deploy and operate the publisher's own apps from uploaded `docker save` archives (tag: publisher). Neither is OAuth.",
        },
        "externalDocs": {"description": "Human-readable docs and publisher deploy quickstart (public)", "url": super::docs::docs_url(base)},
        "servers": [{"url": base}],
        "components": {
            "securitySchemes": {
                "serviceToken": {"type": "http", "scheme": "bearer", "bearerFormat": "rbp_ service token"},
                "publisherToken": {"type": "http", "scheme": "bearer", "bearerFormat": "rbpub_ publisher token"},
            },
            "schemas": {
                "Error": {"type": "object", "properties": {"error": {"type": "object", "properties": {"code": {"type": "string"}, "message": {"type": "string"}}}}},
                "AiPolicyPatch": {"type": "object", "additionalProperties": false, "properties": {
                    "enabled": {"type": "boolean"},
                    "default_model": {"type": "string", "enum": AI_KNOWN_MODELS},
                    "models": {"type": "array", "items": {"type": "string", "enum": AI_KNOWN_MODELS}, "minItems": 1},
                    "max_input_chars": {"type": "integer", "minimum": 1, "maximum": AI_MAX_INPUT_CHARS_CEILING},
                    "max_output_tokens": {"type": "integer", "minimum": 1, "maximum": AI_MAX_OUTPUT_TOKENS_CEILING},
                    "user_daily_requests": {"type": "integer", "minimum": 1, "maximum": AI_USER_DAILY_CEILING},
                    "app_daily_requests": {"type": "integer", "minimum": 1, "maximum": AI_APP_DAILY_CEILING},
                    "public_policy": {"type": ["string", "null"], "enum": [AI_PUBLIC_POLICY_SIGNED_IN_QUOTA, null]},
                }},
                "ReleaseManifest": super::publisher::manifest_schema(),
                "AppRequest": {"type": "object", "required": ["name", "title", "kind", "target", "visibility"], "additionalProperties": false, "properties": {
                    "name": {"type": "string", "pattern": "^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$"},
                    "title": {"type": "string", "maxLength": 80},
                    "description": {"type": "string", "maxLength": 500},
                    "kind": {"type": "string", "enum": ["static", "proxy"]},
                    "target": {"type": "string", "description": "static: absolute root; proxy: 127.0.0.1:PORT"},
                    "visibility": {"type": "string", "enum": ["private", "public_unlisted", "public_listed"]},
                    "identity": {"type": "string", "enum": ["platform"], "description": "required for private apps: the app has no login of its own"},
                    "note": {"type": "string", "maxLength": 500},
                }},
            },
        },
        "paths": {
            "/api/platform/v1": {"get": {"summary": "Capabilities document (public)", "security": [], "responses": {"200": {"description": "OK"}}}},
            "/docs": {"get": {"summary": "Human-readable docs landing with the publisher quickstart (public HTML)", "security": [], "responses": {"200": {"description": "OK", "content": {"text/html": {}}}}}},
            "/api/platform/v1/whoami": {"get": op("Token identity, owner and scopes", "any", json!({}))},
            "/api/platform/v1/apps": {"get": op("Apps this token reaches", "apps:read", json!({}))},
            "/api/platform/v1/apps/{name}": {"get": op("One app", "apps:read", json!({"parameters": name}))},
            "/api/platform/v1/apps/{name}/ai": {
                "get": op("AI policy and today's usage", "ai:read", json!({"parameters": name})),
                "patch": op("Update the AI policy within platform ceilings", "ai:write", json!({"parameters": name, "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/AiPolicyPatch"}}}}})),
            },
            "/api/platform/v1/apps/{name}/route": {"get": op("Preview the generated Caddy route (read-only)", "routes:read", json!({"parameters": name}))},
            "/api/platform/v1/app-requests": {
                "get": op("This owner's registration requests", "apps:request", json!({})),
                "post": op("Request an app registration (an operator approves with the CLI)", "apps:request", json!({"requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/AppRequest"}}}}})),
            },
            "/api/platform/v1/release": {"get": op("Release status: version, schema, AI broker reachability", "release:read", json!({}))},
            "/api/platform/v1/mcp": {"post": op("MCP JSON-RPC endpoint (tools mirror this API)", "any", json!({}))},
        },
    });
    let mut doc = doc;
    if let (Some(paths), Value::Object(extra)) = (
        doc["paths"].as_object_mut(),
        super::publisher::openapi_paths(),
    ) {
        paths.extend(extra);
    }
    let mut r = json_ok_status(StatusCode::OK, &doc);
    r.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );
    r
}

// ------------------------------------------------------------- operations

fn app_json(s: &S, app: &App) -> Value {
    json!({
        "name": app.name,
        "title": app.title,
        "description": app.description,
        "url": app.url(&s.cfg.domain),
        "launcher": format!("{}/{}", s.cfg.public_base, app.name),
        "kind": app.kind.as_str(),
        "visibility": app.visibility.as_str(),
        "enabled": app.enabled,
        "identity": app.identity.as_str(),
        "ai": app.ai.to_json(),
    })
}

fn op_whoami(c: &Caller) -> Value {
    json!({
        "token": c.token.name,
        "owner": c.owner.name,
        "scopes": c.token.scopes,
        "apps": if c.token.apps.is_empty() { json!("all apps owned by the owner") } else { json!(c.token.apps) },
        "expires_at": c.token.expires_at,
    })
}

fn op_apps(s: &S, c: &Caller) -> Result<Value, ApiError> {
    c.need("apps:read")?;
    let apps = s.store.list_apps().map_err(|_| ApiError::internal())?;
    Ok(json!({
        "apps": apps.iter().filter(|a| c.token.reaches(a)).map(|a| app_json(s, a)).collect::<Vec<_>>()
    }))
}

fn op_app(s: &S, c: &Caller, name: &str) -> Result<Value, ApiError> {
    c.need("apps:read")?;
    Ok(app_json(s, &c.app(s, name)?))
}

fn op_ai(s: &S, c: &Caller, name: &str) -> Result<Value, ApiError> {
    c.need("ai:read")?;
    let app = c.app(s, name)?;
    let (requests, users) = s
        .store
        .ai_usage_today(app.id)
        .map_err(|_| ApiError::internal())?;
    Ok(json!({
        "app": app.name,
        "policy": app.ai.to_json(),
        "endpoint": format!("https://{}{AI_CHAT_PATH}", app.host(&s.cfg.domain)),
        "usage_today_utc": {"requests": requests, "users": users},
    }))
}

/// Apply a partial policy update. Unknown fields are refused so a typo can
/// never be silently ignored.
fn op_ai_patch(s: &S, c: &Caller, name: &str, patch: &Value) -> Result<Value, ApiError> {
    c.need("ai:write")?;
    let app = c.app(s, name)?;
    let obj = patch
        .as_object()
        .ok_or_else(|| ApiError::invalid("body must be a JSON object"))?;
    let mut p = app.ai.clone();
    for (k, v) in obj {
        let int = |v: &Value| {
            v.as_i64()
                .ok_or_else(|| ApiError::invalid(format!("{k} must be an integer")))
        };
        match k.as_str() {
            "enabled" => {
                p.enabled = v
                    .as_bool()
                    .ok_or_else(|| ApiError::invalid("enabled must be a boolean"))?
            }
            "default_model" => {
                p.default_model = v
                    .as_str()
                    .ok_or_else(|| ApiError::invalid("default_model must be a string"))?
                    .to_string()
            }
            "models" => {
                p.models = v
                    .as_array()
                    .ok_or_else(|| ApiError::invalid("models must be an array of strings"))?
                    .iter()
                    .map(|m| {
                        m.as_str()
                            .map(str::to_string)
                            .ok_or_else(|| ApiError::invalid("models must be an array of strings"))
                    })
                    .collect::<Result<_, _>>()?
            }
            "max_input_chars" => p.max_input_chars = int(v)?,
            "max_output_tokens" => p.max_output_tokens = int(v)?,
            "user_daily_requests" => p.user_daily_requests = int(v)?,
            "app_daily_requests" => p.app_daily_requests = int(v)?,
            "public_policy" => {
                p.public_policy = match v {
                    Value::Null => None,
                    Value::String(x) => Some(x.clone()),
                    _ => return Err(ApiError::invalid("public_policy must be a string or null")),
                }
            }
            other => {
                return Err(ApiError::invalid(format!(
                    "unknown field '{other}' (provider is fixed to chatmock in v1)"
                )));
            }
        }
    }
    let updated = s.store.set_app_ai(app.id, &p).map_err(|e| match e {
        crate::store::StoreError::Invalid(m) => ApiError::invalid(m),
        crate::store::StoreError::Conflict(m) => ApiError::new(StatusCode::CONFLICT, "conflict", m),
        _ => ApiError::internal(),
    })?;
    c.audit(
        s,
        "app.ai",
        &app.name,
        &format!(
            "enabled={} default={} models={} in={} out={} user/day={} app/day={} public={}",
            updated.ai.enabled,
            updated.ai.default_model,
            updated.ai.models_csv(),
            updated.ai.max_input_chars,
            updated.ai.max_output_tokens,
            updated.ai.user_daily_requests,
            updated.ai.app_daily_requests,
            updated.ai.public_policy.as_deref().unwrap_or("-")
        ),
    );
    Ok(json!({"app": updated.name, "policy": updated.ai.to_json()}))
}

fn op_route(s: &S, c: &Caller, name: &str) -> Result<Value, ApiError> {
    c.need("routes:read")?;
    let app = c.app(s, name)?;
    let text = crate::render::render(std::slice::from_ref(&app), &s.cfg.routes)
        .map_err(ApiError::invalid)?;
    Ok(json!({
        "app": app.name,
        "caddy": text,
        "note": "read-only preview; routes are applied only by an operator (validated, backed up, rollback retained)",
    }))
}

fn request_json(r: &crate::store::AppRequest) -> Value {
    json!({
        "id": r.id, "name": r.name, "title": r.title, "kind": r.kind.as_str(),
        "target": r.target, "visibility": r.visibility.as_str(), "status": r.status,
        "created_at": r.created_at, "decided_at": r.decided_at, "decision_note": r.decision_note,
    })
}

fn op_requests(s: &S, c: &Caller) -> Result<Value, ApiError> {
    c.need("apps:request")?;
    let list = s
        .store
        .list_app_requests(Some(c.owner.id))
        .map_err(|_| ApiError::internal())?;
    Ok(json!({"requests": list.iter().map(request_json).collect::<Vec<_>>()}))
}

fn op_request_create(s: &S, c: &Caller, body: &Value) -> Result<Value, ApiError> {
    c.need("apps:request")?;
    let obj = body
        .as_object()
        .ok_or_else(|| ApiError::invalid("body must be a JSON object"))?;
    for k in obj.keys() {
        if !matches!(
            k.as_str(),
            "name"
                | "title"
                | "description"
                | "kind"
                | "target"
                | "visibility"
                | "identity"
                | "note"
        ) {
            return Err(ApiError::invalid(format!("unknown field '{k}'")));
        }
    }
    let field = |k: &str| obj.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let kind = AppKind::parse(&field("kind"))
        .ok_or_else(|| ApiError::invalid("kind must be static or proxy"))?;
    let visibility = Visibility::parse(&field("visibility")).ok_or_else(|| {
        ApiError::invalid("visibility must be private, public_unlisted or public_listed")
    })?;
    if visibility == Visibility::Private && field("identity") != "platform" {
        return Err(ApiError::invalid(
            crate::store::PRIVATE_NEEDS_PLATFORM_IDENTITY,
        ));
    }
    if !c.token.apps.is_empty() {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "insufficient_scope",
            "an app-restricted token cannot request new apps",
        ));
    }
    let r = s
        .store
        .create_app_request(
            &field("name"),
            &field("title"),
            &field("description"),
            kind,
            &field("target"),
            visibility,
            c.owner.id,
            Some(c.token.id),
            &field("note"),
        )
        .map_err(|e| match e {
            crate::store::StoreError::Invalid(m) => ApiError::invalid(m),
            crate::store::StoreError::Conflict(m) => {
                ApiError::new(StatusCode::CONFLICT, "conflict", m)
            }
            _ => ApiError::internal(),
        })?;
    c.audit(s, "app.request", &r.name, &format!("id={}", r.id));
    Ok(json!({
        "request": request_json(&r),
        "next": "an operator reviews it and runs `repobox-platform app requests approve <id>`; routes are then rendered and applied by the operator",
    }))
}

async fn op_release(s: &S, c: &Caller) -> Result<Value, ApiError> {
    c.need("release:read")?;
    let schema = s.store.schema_version().unwrap_or_else(|_| "?".into());
    let ai = match s.cfg.ai.as_deref() {
        None => json!({"configured": false}),
        Some(b) => {
            let r = b
                .upstream
                .send(
                    axum::http::Method::GET,
                    "/v1/models",
                    &[("authorization", b.secret.bearer())],
                    None,
                    std::time::Duration::from_secs(10),
                )
                .await;
            match r {
                Ok(reply) if reply.status.is_success() => {
                    let models: Vec<String> = serde_json::from_slice::<Value>(&reply.body)
                        .ok()
                        .and_then(|v| {
                            v["data"].as_array().map(|a| {
                                a.iter()
                                    .filter_map(|m| m["id"].as_str().map(str::to_string))
                                    .collect()
                            })
                        })
                        .unwrap_or_default();
                    json!({"configured": true, "reachable": true, "models": models})
                }
                Ok(reply) => {
                    json!({"configured": true, "reachable": false, "status": reply.status.as_u16()})
                }
                Err(e) => json!({"configured": true, "reachable": false, "error": e.code}),
            }
        }
    };
    Ok(json!({
        "api_version": API_VERSION,
        "software_version": build_version(),
        "commit": build_commit(),
        "schema_version": schema,
        "ai_broker": ai,
    }))
}

// --------------------------------------------------------------- handlers

pub async fn whoami(State(s): State<S>, headers: HeaderMap) -> Response {
    reply(caller(&s, &headers).map(|c| op_whoami(&c)))
}

pub async fn apps(State(s): State<S>, headers: HeaderMap) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_apps(&s, &c)))
}

pub async fn app(State(s): State<S>, headers: HeaderMap, Path(name): Path<String>) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_app(&s, &c, &name)))
}

pub async fn app_ai(State(s): State<S>, headers: HeaderMap, Path(name): Path<String>) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_ai(&s, &c, &name)))
}

pub async fn app_ai_patch(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    body: Option<Json<Value>>,
) -> Response {
    reply(caller(&s, &headers).and_then(|c| {
        let Some(Json(v)) = body else {
            return Err(ApiError::invalid(
                "body must be JSON (Content-Type: application/json)",
            ));
        };
        op_ai_patch(&s, &c, &name, &v)
    }))
}

pub async fn app_route(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_route(&s, &c, &name)))
}

pub async fn app_requests(State(s): State<S>, headers: HeaderMap) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_requests(&s, &c)))
}

pub async fn app_request_create(
    State(s): State<S>,
    headers: HeaderMap,
    body: Option<Json<Value>>,
) -> Response {
    reply(caller(&s, &headers).and_then(|c| {
        let Some(Json(v)) = body else {
            return Err(ApiError::invalid(
                "body must be JSON (Content-Type: application/json)",
            ));
        };
        op_request_create(&s, &c, &v)
    }))
}

pub async fn release(State(s): State<S>, headers: HeaderMap) -> Response {
    match caller(&s, &headers) {
        Ok(c) => reply(op_release(&s, &c).await),
        Err(e) => e.into_response(),
    }
}

pub async fn api_not_found(State(s): State<S>) -> Response {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        format!(
            "unknown API path; docs: {}",
            super::docs::docs_url(&s.cfg.public_base)
        ),
    )
    .into_response()
}

// -------------------------------------------------------------------- MCP

fn tool(name: &str, description: &str, scope: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": format!("{description} (scope: {scope})"),
        "inputSchema": {"type": "object", "properties": props, "required": required, "additionalProperties": false},
    })
}

pub fn mcp_tools() -> Vec<Value> {
    let name = json!({"name": {"type": "string", "description": "app name (DNS label)"}});
    let mut patch_props = json!({"name": {"type": "string"}});
    for (k, v) in [
        ("enabled", json!({"type": "boolean"})),
        (
            "default_model",
            json!({"type": "string", "enum": AI_KNOWN_MODELS}),
        ),
        (
            "models",
            json!({"type": "array", "items": {"type": "string", "enum": AI_KNOWN_MODELS}}),
        ),
        (
            "max_input_chars",
            json!({"type": "integer", "minimum": 1, "maximum": AI_MAX_INPUT_CHARS_CEILING}),
        ),
        (
            "max_output_tokens",
            json!({"type": "integer", "minimum": 1, "maximum": AI_MAX_OUTPUT_TOKENS_CEILING}),
        ),
        (
            "user_daily_requests",
            json!({"type": "integer", "minimum": 1, "maximum": AI_USER_DAILY_CEILING}),
        ),
        (
            "app_daily_requests",
            json!({"type": "integer", "minimum": 1, "maximum": AI_APP_DAILY_CEILING}),
        ),
        (
            "public_policy",
            json!({"type": ["string", "null"], "enum": [AI_PUBLIC_POLICY_SIGNED_IN_QUOTA, null]}),
        ),
    ] {
        patch_props[k] = v;
    }
    vec![
        tool(
            "whoami",
            "Show the service token's owner, scopes and app restriction",
            "any",
            json!({}),
            &[],
        ),
        tool(
            "list_apps",
            "List the apps this token reaches, with identity contract and AI policy",
            "apps:read",
            json!({}),
            &[],
        ),
        tool(
            "get_app",
            "Show one app",
            "apps:read",
            name.clone(),
            &["name"],
        ),
        tool(
            "get_ai_policy",
            "Show an app's AI policy, endpoint and today's usage counters",
            "ai:read",
            name.clone(),
            &["name"],
        ),
        tool(
            "update_ai_policy",
            "Change an app's AI policy within platform ceilings; public apps need public_policy 'signed-in-quota' to enable AI",
            "ai:write",
            patch_props,
            &["name"],
        ),
        tool(
            "preview_route",
            "Render the generated Caddy route of an app (read-only; applying is operator-only)",
            "routes:read",
            name,
            &["name"],
        ),
        tool(
            "request_app_registration",
            "File an app registration request for an operator to approve with the CLI",
            "apps:request",
            json!({
                "name": {"type": "string"}, "title": {"type": "string"}, "description": {"type": "string"},
                "kind": {"type": "string", "enum": ["static", "proxy"]},
                "target": {"type": "string", "description": "static: absolute root; proxy: 127.0.0.1:PORT"},
                "visibility": {"type": "string", "enum": ["private", "public_unlisted", "public_listed"]},
                "identity": {"type": "string", "enum": ["platform"]},
                "note": {"type": "string"},
            }),
            &["name", "title", "kind", "target", "visibility"],
        ),
        tool(
            "list_app_requests",
            "List this owner's registration requests and their status",
            "apps:request",
            json!({}),
            &[],
        ),
        tool(
            "release_status",
            "Control plane version, schema and AI broker reachability",
            "release:read",
            json!({}),
            &[],
        ),
    ]
}

fn rpc_result(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn rpc_error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

pub async fn mcp_get() -> Response {
    let mut r = (StatusCode::METHOD_NOT_ALLOWED, "").into_response();
    r.headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST"));
    r
}

pub async fn mcp(State(s): State<S>, headers: HeaderMap, body: Option<Json<Value>>) -> Response {
    // Publisher tokens get the publisher tool set (deploy/operate own apps).
    let is_publisher = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with(&format!("Bearer {}", crate::publisher::TOKEN_PREFIX)));
    if is_publisher {
        return mcp_publisher(s, headers, body).await;
    }
    // Every MCP call needs the bearer; unauthenticated callers get the plain
    // 401 (with WWW-Authenticate) rather than a JSON-RPC envelope.
    let c = match caller(&s, &headers) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let Some(Json(msg)) = body else {
        return json_ok_status(
            StatusCode::BAD_REQUEST,
            &rpc_error(
                &Value::Null,
                -32700,
                "parse error: expected a JSON-RPC object",
            ),
        );
    };
    if msg.is_array() {
        return json_ok_status(
            StatusCode::BAD_REQUEST,
            &rpc_error(&Value::Null, -32600, "batches are not supported"),
        );
    }
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    if msg.get("id").is_none() {
        // Notification (e.g. notifications/initialized): accepted, no body.
        return (StatusCode::ACCEPTED, "").into_response();
    }
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    let out = match method {
        "initialize" => rpc_result(
            &id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "repobox-platform", "version": build_version()},
                "instructions": format!("repo.box platform control plane. Tools are owner-scoped by the service token. App registration is a request an operator approves; applying routes, users, grants and provider credentials are never available here. Docs: {}", super::docs::docs_url(&s.cfg.public_base)),
            }),
        ),
        "ping" => rpc_result(&id, json!({})),
        "tools/list" => rpc_result(&id, json!({"tools": mcp_tools()})),
        "tools/call" => {
            let tool = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let name_arg = args
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let r: Result<Value, ApiError> = match tool {
                "whoami" => Ok(op_whoami(&c)),
                "list_apps" => op_apps(&s, &c),
                "get_app" => op_app(&s, &c, &name_arg),
                "get_ai_policy" => op_ai(&s, &c, &name_arg),
                "update_ai_policy" => {
                    let mut patch = args.clone();
                    if let Some(o) = patch.as_object_mut() {
                        o.remove("name");
                    }
                    op_ai_patch(&s, &c, &name_arg, &patch)
                }
                "preview_route" => op_route(&s, &c, &name_arg),
                "request_app_registration" => op_request_create(&s, &c, &args),
                "list_app_requests" => op_requests(&s, &c),
                "release_status" => op_release(&s, &c).await,
                _ => {
                    return json_ok_status(
                        StatusCode::OK,
                        &rpc_error(&id, -32602, &format!("unknown tool '{tool}'")),
                    );
                }
            };
            match r {
                Ok(v) => rpc_result(
                    &id,
                    json!({"content": [{"type": "text", "text": v.to_string()}], "structuredContent": v, "isError": false}),
                ),
                Err(e) => rpc_result(
                    &id,
                    json!({"content": [{"type": "text", "text": e.body().to_string()}], "structuredContent": e.body(), "isError": true}),
                ),
            }
        }
        _ => rpc_error(&id, -32601, &format!("method '{method}' not found")),
    };
    json_ok_status(StatusCode::OK, &out)
}

async fn mcp_publisher(s: S, headers: HeaderMap, body: Option<Json<Value>>) -> Response {
    use super::publisher as p;
    let c = match p::caller(&s, &headers) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let Some(Json(msg)) = body else {
        return json_ok_status(
            StatusCode::BAD_REQUEST,
            &rpc_error(
                &Value::Null,
                -32700,
                "parse error: expected a JSON-RPC object",
            ),
        );
    };
    if msg.is_array() {
        return json_ok_status(
            StatusCode::BAD_REQUEST,
            &rpc_error(&Value::Null, -32600, "batches are not supported"),
        );
    }
    let Some(id) = msg.get("id").cloned() else {
        return (StatusCode::ACCEPTED, "").into_response();
    };
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    let out = match method {
        "initialize" => rpc_result(
            &id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "repobox-platform-publisher", "version": build_version()},
                "instructions": format!("repo.box publisher. Deploying is a direct HTTPS multipart upload of a Docker image archive (`docker save` output) plus a JSON manifest to POST {base}/api/platform/v1/publisher/releases; MCP carries no image bytes, so call how_to_deploy for the exact request. repo.box does not clone, pull or build anything. The app is private with the platform identity and the AI endpoint on, and only this publisher can see or change it. Poll get_release until done, then share the launcher_url. Docs: {docs}", base = s.cfg.public_base, docs = super::docs::docs_url(&s.cfg.public_base)),
            }),
        ),
        "ping" => rpc_result(&id, json!({})),
        "tools/list" => rpc_result(&id, json!({"tools": p::mcp_tools()})),
        "tools/call" => {
            let tool = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match p::mcp_call(&s, &c, tool, &args).await {
                None => rpc_error(&id, -32602, &format!("unknown tool '{tool}'")),
                Some(Ok(v)) => rpc_result(
                    &id,
                    json!({"content": [{"type": "text", "text": v.to_string()}], "structuredContent": v, "isError": false}),
                ),
                Some(Err(e)) => rpc_result(
                    &id,
                    json!({"content": [{"type": "text", "text": e.to_string()}], "structuredContent": e, "isError": true}),
                ),
            }
        }
        _ => rpc_error(&id, -32601, &format!("method '{method}' not found")),
    };
    json_ok_status(StatusCode::OK, &out)
}
