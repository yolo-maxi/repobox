//! Tiny HTML helpers: escaping, the page shell and shared fragments. No
//! template engine, no client-side framework; every page is server-rendered
//! and works without JavaScript.

use crate::model::{App, User, Visibility};

pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub struct Shell<'a> {
    pub title: &'a str,
    pub user: Option<&'a User>,
    pub active: &'a str,
    /// Set (to the auth site's public base) when the page is served on an app
    /// host by the gate: CSS is inlined because `/assets/*` is gated there,
    /// and the brand links back to the auth site instead of `/`.
    pub standalone: Option<&'a str>,
}

pub fn page(shell: &Shell<'_>, body: &str) -> String {
    let (brand_href, stylesheet) = match shell.standalone {
        Some(base) => (
            format!("{base}/"),
            format!("<style>{}</style>", super::css::CSS),
        ),
        None => (
            "/".to_string(),
            "<link rel=\"stylesheet\" href=\"/assets/app.css\">".to_string(),
        ),
    };
    let nav = if shell.standalone.is_some() {
        String::new()
    } else {
        let mut items = vec![("/", "Directory", "directory")];
        if let Some(u) = shell.user {
            items.push(("/me", "Account", "me"));
            if u.is_admin() {
                items.push(("/admin/users", "Users", "users"));
                items.push(("/admin/audit", "Audit", "audit"));
            }
        }
        items
            .iter()
            .map(|(href, label, key)| {
                let cls = if *key == shell.active {
                    " class=\"active\""
                } else {
                    ""
                };
                format!("<a href=\"{href}\"{cls}>{label}</a>")
            })
            .collect::<String>()
    };
    let who = if shell.standalone.is_some() {
        String::new()
    } else {
        match shell.user {
            Some(u) => format!(
                "<span class=\"who\"><span class=\"avatar\">{}</span>{}{}</span>",
                esc(&initial(&u.display_name)),
                esc(&u.name),
                if u.is_admin() {
                    " <span class=\"badge admin\">admin</span>"
                } else {
                    ""
                }
            ),
            None => "<span class=\"who\">not signed in</span>".to_string(),
        }
    };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>{title} · auth.repo.box</title>
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Ccircle cx='16' cy='16' r='14' fill='%230a1628'/%3E%3Ccircle cx='16' cy='16' r='7' fill='%234fc3f7'/%3E%3C/svg%3E">
{stylesheet}
</head>
<body>
<header class="top"><div class="wrap">
<a class="brand" href="{brand_href}"><span class="dot"></span>repo.box <small>/ auth</small></a>
<nav class="main">{nav}</nav>
{who}
</div></header>
<main><div class="wrap">
{body}
</div></main>
<footer><div class="wrap"><span>repo.box platform control plane</span><span class="muted">no passwords · no email · device links only</span></div></footer>
</body>
</html>"#,
        title = esc(shell.title),
        brand_href = brand_href,
        stylesheet = stylesheet,
    )
}

pub fn initial(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into())
}

pub fn status_page(
    shell: &Shell<'_>,
    icon: &str,
    heading: &str,
    text: &str,
    actions: &str,
) -> String {
    page(
        shell,
        &format!(
            "<div class=\"status-page\"><div class=\"icon\">{icon}</div><h1>{}</h1><p>{}</p><div class=\"row\" style=\"justify-content:center\">{actions}</div></div>",
            esc(heading),
            esc(text)
        ),
    )
}

pub fn vis_badge(v: Visibility) -> String {
    format!(
        "<span class=\"badge {}\">{}</span>",
        v.as_str(),
        esc(v.label())
    )
}

pub fn flash(ok: Option<&str>, err: Option<&str>) -> String {
    let mut s = String::new();
    if let Some(m) = ok {
        s.push_str(&format!("<div class=\"flash ok\">{}</div>", esc(m)));
    }
    if let Some(m) = err {
        s.push_str(&format!("<div class=\"flash err\">{}</div>", esc(m)));
    }
    s
}

pub fn app_card(app: &App, domain: &str, signed_in: bool, manage: bool) -> String {
    let host = app.host(domain);
    let open = if signed_in {
        format!(
            "<a class=\"btn primary small\" href=\"/{}\">Open</a>",
            esc(&app.name)
        )
    } else {
        format!(
            "<a class=\"btn primary small\" href=\"{}\">Open</a>",
            esc(&app.url(domain))
        )
    };
    let manage = if manage {
        format!(
            "<a class=\"btn small\" href=\"/apps/{}\">Manage</a>",
            esc(&app.name)
        )
    } else {
        String::new()
    };
    let off = if app.enabled {
        ""
    } else {
        "<span class=\"badge off\">disabled</span>"
    };
    let desc = if app.description.is_empty() {
        String::new()
    } else {
        format!("<p class=\"desc\">{}</p>", esc(&app.description))
    };
    format!(
        "<div class=\"card\"><div class=\"title\"><a href=\"/{name}\">{title}</a>{badge}</div><div class=\"host\">{host}</div>{desc}<div class=\"actions\">{open}{manage}{off}</div></div>",
        name = esc(&app.name),
        title = esc(&app.title),
        badge = vis_badge(app.visibility),
        host = esc(&host),
    )
}

pub fn fmt_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

pub fn fmt_rel(now: i64, ts: i64) -> String {
    let d = ts - now;
    let (abs, suffix, prefix) = if d >= 0 {
        (d, "", "in ")
    } else {
        (-d, " ago", "")
    };
    let s = if abs < 60 {
        format!("{abs}s")
    } else if abs < 3600 {
        format!("{}m", abs / 60)
    } else if abs < 86400 {
        format!("{}h", abs / 3600)
    } else {
        format!("{}d", abs / 86400)
    };
    format!("{prefix}{s}{suffix}")
}
