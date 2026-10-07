//! auth.repo.box as an installable PWA: the web manifest, the service worker,
//! the install/session client script, the static offline page and the icons.
//!
//! The service worker only ever caches the static shell listed in [`SHELL`]
//! (stylesheet, this script, the offline page, icons). Pages, redirects,
//! cookies, launch codes, invite tokens and API answers always go to the
//! network and are never written to any cache; offline, a navigation gets an
//! honest "you are offline" page instead of stale private data.

use std::sync::OnceLock;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};

use super::{S, current_user};

pub const THEME_COLOR: &str = "#0a1628";
pub const MANIFEST_PATH: &str = "/manifest.webmanifest";
pub const SW_PATH: &str = "/sw.js";
pub const SCRIPT_PATH: &str = "/assets/pwa.js";
pub const OFFLINE_PATH: &str = "/assets/offline.html";
pub const SESSION_PATH: &str = "/api/session";

/// `(file name under /assets/icons/, content type, bytes)`.
pub const ICONS: &[(&str, &str, &[u8])] = &[
    (
        "icon.svg",
        "image/svg+xml",
        include_bytes!("../../assets/pwa/icon.svg"),
    ),
    (
        "icon-192.png",
        "image/png",
        include_bytes!("../../assets/pwa/icon-192.png"),
    ),
    (
        "icon-512.png",
        "image/png",
        include_bytes!("../../assets/pwa/icon-512.png"),
    ),
    (
        "maskable-192.png",
        "image/png",
        include_bytes!("../../assets/pwa/maskable-192.png"),
    ),
    (
        "maskable-512.png",
        "image/png",
        include_bytes!("../../assets/pwa/maskable-512.png"),
    ),
    (
        "apple-touch-icon.png",
        "image/png",
        include_bytes!("../../assets/pwa/apple-touch-icon.png"),
    ),
];

/// The only URLs the service worker caches (exact paths, no query).
pub const SHELL: &[&str] = &[
    "/assets/app.css",
    SCRIPT_PATH,
    OFFLINE_PATH,
    "/assets/icons/icon.svg",
    "/assets/icons/icon-192.png",
    "/assets/icons/maskable-192.png",
    "/assets/icons/apple-touch-icon.png",
];

fn icon(name: &str) -> Option<(&'static str, &'static [u8])> {
    ICONS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, ct, b)| (*ct, *b))
}

pub fn manifest_json() -> serde_json::Value {
    let icon = |src: &str, sizes: &str, ty: &str, purpose: &str| serde_json::json!({ "src": src, "sizes": sizes, "type": ty, "purpose": purpose });
    serde_json::json!({
        "id": "/",
        "name": "repo.box auth",
        "short_name": "repo.box",
        "description": "Your repo.box apps, account and signed-in devices.",
        "lang": "en",
        "start_url": "/",
        "scope": "/",
        "display": "standalone",
        "background_color": THEME_COLOR,
        "theme_color": THEME_COLOR,
        "icons": [
            icon("/assets/icons/icon.svg", "any", "image/svg+xml", "any"),
            icon("/assets/icons/icon-192.png", "192x192", "image/png", "any"),
            icon("/assets/icons/icon-512.png", "512x512", "image/png", "any"),
            icon("/assets/icons/maskable-192.png", "192x192", "image/png", "maskable"),
            icon("/assets/icons/maskable-512.png", "512x512", "image/png", "maskable"),
        ],
    })
}

/// Content hash of everything the worker serves from cache (and of the worker
/// itself), so any change to the shell ships a byte-different worker, which
/// the browser installs and which drops the previous cache.
pub fn version() -> &'static str {
    static V: OnceLock<String> = OnceLock::new();
    V.get_or_init(|| {
        let mut h = Sha256::new();
        for part in [
            super::css::CSS.as_bytes(),
            SCRIPT.as_bytes(),
            OFFLINE_HTML.as_bytes(),
            SW_TEMPLATE.as_bytes(),
            manifest_json().to_string().as_bytes(),
        ] {
            h.update((part.len() as u64).to_le_bytes());
            h.update(part);
        }
        for (name, _, bytes) in ICONS {
            h.update(name.as_bytes());
            h.update(bytes);
        }
        h.finalize()[..6]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    })
}

pub fn service_worker() -> String {
    let shell = serde_json::to_string(SHELL).unwrap();
    SW_TEMPLATE
        .replace("__VERSION__", version())
        .replace("__SHELL__", &shell)
        .replace("__OFFLINE__", OFFLINE_PATH)
}

fn asset(ct: &'static str, cache: &'static str, body: impl IntoResponse) -> Response {
    (
        [
            (header::CONTENT_TYPE, ct),
            (header::CACHE_CONTROL, cache),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

pub async fn manifest() -> Response {
    asset(
        "application/manifest+json",
        "no-cache",
        manifest_json().to_string(),
    )
}

/// Never cached by HTTP: the browser must see a new worker as soon as one is
/// deployed (registration also sets `updateViaCache: 'none'`).
pub async fn sw() -> Response {
    asset(
        "text/javascript; charset=utf-8",
        "no-store",
        service_worker(),
    )
}

pub async fn script() -> Response {
    asset(
        "text/javascript; charset=utf-8",
        "public, max-age=300",
        SCRIPT,
    )
}

pub async fn offline() -> Response {
    asset(
        "text/html; charset=utf-8",
        "public, max-age=300",
        OFFLINE_HTML,
    )
}

pub async fn icon_file(Path(name): Path<String>) -> Response {
    match icon(&name) {
        Some((ct, bytes)) => asset(ct, "public, max-age=86400", bytes),
        None => asset_missing().await,
    }
}

/// `/apple-touch-icon.png` at the root: iOS probes it even with the link tag.
pub async fn apple_touch_icon() -> Response {
    let (ct, bytes) = icon("apple-touch-icon.png").unwrap();
    asset(ct, "public, max-age=86400", bytes)
}

/// Missing static assets are a plain 404, never an HTML page a worker or
/// browser could mistake for the asset.
pub async fn asset_missing() -> Response {
    (
        StatusCode::NOT_FOUND,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        "not found\n",
    )
        .into_response()
}

/// `GET /api/session`: is this device signed in, and as which user id. Read
/// by the page script when an installed app resumes, so a revoked or expired
/// device never keeps showing a signed-in page. Same-origin only (no CORS).
pub async fn session(State(s): State<S>, headers: HeaderMap) -> Response {
    let user = current_user(&s, &headers).map(|(_, u)| u);
    let mut r = Json(serde_json::json!({
        "signed_in": user.is_some(),
        "user_id": user.map(|u| u.id),
    }))
    .into_response();
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    r
}

/// Head tags for every auth.repo.box page (not the gate's app-host pages).
pub fn head_tags() -> String {
    format!(
        "<link rel=\"manifest\" href=\"{MANIFEST_PATH}\">\n<meta name=\"theme-color\" content=\"{THEME_COLOR}\">\n<link rel=\"icon\" href=\"/assets/icons/icon.svg\" type=\"image/svg+xml\">\n<link rel=\"apple-touch-icon\" href=\"/assets/icons/apple-touch-icon.png\">\n<meta name=\"apple-mobile-web-app-title\" content=\"repo.box\">\n<script src=\"{SCRIPT_PATH}\" defer></script>"
    )
}

/// The install action (shown by the script only once the browser has really
/// offered installation) and the iOS hint (shown only on iOS outside the
/// installed app). Both start hidden, so without the script nothing claims
/// installation is possible.
pub const HEADER_INSTALL: &str = "<button type=\"button\" class=\"btn small primary\" id=\"pwa-install\" hidden>Install app</button>";

pub const IOS_HINT: &str = "<div class=\"pwa-ios\" id=\"pwa-ios\" hidden><div class=\"wrap\"><span>Install on iPhone or iPad: tap <strong>Share</strong>, then <strong>Add to Home Screen</strong>.</span><button type=\"button\" class=\"btn small\" id=\"pwa-ios-close\">Got it</button></div></div><p class=\"sr-only\" id=\"pwa-status\" role=\"status\" aria-live=\"polite\"></p>";

/// Account-page panel describing this device's install state truthfully.
pub const INSTALL_PANEL: &str = "<div class=\"panel\" id=\"pwa-panel\"><h3>Install on this device</h3><p class=\"muted\" id=\"pwa-state\" style=\"margin:0\">This browser has not offered to install auth.repo.box.</p></div>";

pub const OFFLINE_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<meta name="theme-color" content="#0a1628">
<title>Offline · auth.repo.box</title>
<link rel="icon" href="/assets/icons/icon.svg" type="image/svg+xml">
<link rel="stylesheet" href="/assets/app.css">
</head>
<body data-offline="1">
<header class="top"><div class="wrap"><span class="brand"><span class="dot"></span>repo.box <small>/ auth</small></span></div></header>
<main><div class="wrap"><div class="status-page"><div class="icon">⌁</div><h1>You are offline</h1><p>auth.repo.box needs a connection to show your apps, account and devices. Nothing private is stored on this device, so there is nothing to show until you are back online.</p><div class="row" style="justify-content:center"><button class="btn primary" type="button" onclick="location.reload()">Try again</button></div></div></div></main>
</body>
</html>
"##;

const SW_TEMPLATE: &str = r#"// auth.repo.box service worker. Caches only the static shell below; every
// page, redirect and API answer goes to the network and is never stored.
const VERSION = '__VERSION__';
const CACHE = 'rb-auth-shell-' + VERSION;
const SHELL = __SHELL__;
const OFFLINE = '__OFFLINE__';

self.addEventListener('install', (e) => {
  e.waitUntil(
    caches.open(CACHE)
      .then((c) => c.addAll(SHELL.map((u) => new Request(u, { cache: 'reload', credentials: 'omit' }))))
      .then(() => self.skipWaiting(), (err) => caches.delete(CACHE).then(() => { throw err; }))
  );
});

self.addEventListener('activate', (e) => {
  e.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((k) => k.startsWith('rb-auth-') && k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim())
  );
});

function offline() {
  return caches.match(OFFLINE, { cacheName: CACHE }).then((r) => r
    ? new Response(r.body, { status: 503, statusText: 'Offline', headers: { 'Content-Type': 'text/html; charset=utf-8', 'Cache-Control': 'no-store' } })
    : new Response('You are offline.\n', { status: 503, headers: { 'Content-Type': 'text/plain; charset=utf-8', 'Cache-Control': 'no-store' } }));
}

self.addEventListener('fetch', (e) => {
  const req = e.request;
  if (req.method !== 'GET') return;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  if (req.mode === 'navigate') {
    // Always the network; offline gets the honest offline page, never a copy.
    e.respondWith(fetch(req).catch(offline));
    return;
  }
  if (!url.search && SHELL.includes(url.pathname)) {
    // Read-only lookup: never (re)creates a cache, even in a retiring worker.
    e.respondWith(caches.match(url.pathname, { cacheName: CACHE }).then((r) => r || fetch(req)));
  }
  // Anything else is not intercepted: straight to the network, uncached.
});
"#;

pub const SCRIPT: &str = r#"// auth.repo.box: service-worker registration, the install action and the
// session check that keeps an installed app from showing a stale signed-in
// page after its device session was revoked or expired.
(function () {
  'use strict';
  var d = document, n = navigator, root = d.documentElement;
  var standalone = (window.matchMedia && matchMedia('(display-mode: standalone)').matches) || n.standalone === true;
  var ios = /iPad|iPhone|iPod/.test(n.userAgent) || (n.platform === 'MacIntel' && n.maxTouchPoints > 1);
  var deferred = null;
  var state = standalone ? 'standalone' : ios ? 'ios' : 'unsupported';
  var TEXT = {
    standalone: 'You are using the installed app.',
    available: 'This browser offers to install auth.repo.box. Use Install app at the top of the page.',
    accepted: 'Installing auth.repo.box. Open it from your home screen or app list.',
    installed: 'auth.repo.box is installed. Open it from your home screen or app list.',
    dismissed: 'Install dismissed. Your browser may offer it again later, or from its menu.',
    ios: 'On iPhone or iPad: tap Share, then Add to Home Screen. Safari has no install button a page can trigger.',
    unsupported: 'This browser has not offered to install auth.repo.box. It may already be installed, or the browser may not install web apps.'
  };
  function $(id) { return d.getElementById(id); }
  function set(s, announce) {
    state = s;
    root.setAttribute('data-pwa', s);
    var p = $('pwa-state');
    if (p) p.textContent = TEXT[s];
    var b = $('pwa-install');
    if (b) b.hidden = s !== 'available';
    if (announce && $('pwa-status')) $('pwa-status').textContent = TEXT[s];
  }

  if ('serviceWorker' in n) {
    n.serviceWorker.register('/sw.js', { scope: '/', updateViaCache: 'none' })
      .then(function (r) { root.setAttribute('data-sw', 'registered'); return r.update(); })
      .catch(function () { root.setAttribute('data-sw', 'failed'); });
  }

  window.addEventListener('beforeinstallprompt', function (e) {
    e.preventDefault();
    deferred = e;
    set('available');
  });
  window.addEventListener('appinstalled', function () {
    deferred = null;
    set('installed', true);
  });

  function ready() {
    set(state);
    var b = $('pwa-install');
    if (b) b.addEventListener('click', function () {
      var e = deferred;
      deferred = null;
      if (!e) { set('unsupported'); return; }
      b.hidden = true;
      e.prompt();
      e.userChoice.then(function (c) {
        if (state === 'installed') return;
        set(c && c.outcome === 'accepted' ? 'accepted' : 'dismissed', true);
      }, function () { set('dismissed', true); });
    });
    var hint = $('pwa-ios');
    var seen = false;
    try { seen = localStorage.getItem('rb-pwa-ios-hint') === 'seen'; } catch (_) {}
    if (hint && ios && !standalone && !seen) {
      hint.hidden = false;
      $('pwa-ios-close').addEventListener('click', function () {
        hint.hidden = true;
        try { localStorage.setItem('rb-pwa-ios-hint', 'seen'); } catch (_) {}
      });
    }
  }
  if (d.readyState === 'loading') d.addEventListener('DOMContentLoaded', ready); else ready();

  // A page rendered for a signed-in device re-checks the session whenever it
  // comes back into view (an installed app is often resumed, not reloaded).
  // Revoked, expired or switched: start over at the directory, which renders
  // the normal signed-out (or new) state. Offline under the worker: the
  // honest offline page rather than the old signed-in page.
  function recheck() {
    var rendered = d.body && d.body.getAttribute('data-session');
    if (!rendered) return;
    fetch('/api/session', { cache: 'no-store', credentials: 'same-origin' })
      .then(function (r) { if (!r.ok) throw new Error(r.status); return r.json(); })
      .then(function (j) {
        if (String(j.user_id == null ? '' : j.user_id) !== rendered) location.replace('/');
      }, function () {
        if (n.serviceWorker && n.serviceWorker.controller) location.replace('/');
      });
  }
  d.addEventListener('visibilitychange', function () { if (d.visibilityState === 'visible') recheck(); });
  window.addEventListener('pageshow', function (e) { if (e.persisted) recheck(); });
  window.addEventListener('online', recheck);
})();
"#;
