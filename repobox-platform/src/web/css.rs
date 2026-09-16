//! One embedded stylesheet. Tokens follow the repo.box `--bp-*` palette so the
//! control plane reads as part of the same product.

pub const CSS: &str = r#"
:root{--bg:#0a1628;--surface:#0d1f35;--surface2:#112743;--border:rgba(50,100,160,.28);--text:#b8d4e3;--heading:#e8f4fd;--dim:#7a9ab4;--accent:#4fc3f7;--accent2:#81d4fa;--gold:#f0b860;--ok:#6ee7a8;--warn:#f0b860;--bad:#ff7b8a;--radius:12px;--mono:"JetBrains Mono",ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;--sans:system-ui,-apple-system,"Segoe UI",Roboto,Inter,sans-serif}
*{box-sizing:border-box}
html{-webkit-text-size-adjust:100%}
body{margin:0;background:var(--bg);color:var(--text);font-family:var(--sans);line-height:1.5;min-height:100vh;background-image:radial-gradient(1200px 600px at 80% -10%,rgba(79,195,247,.08),transparent 60%),radial-gradient(800px 400px at -10% 110%,rgba(240,184,96,.06),transparent 60%)}
a{color:var(--accent);text-decoration:none}a:hover{text-decoration:underline}
code,.mono{font-family:var(--mono);font-size:.92em}
.wrap{max-width:1040px;margin:0 auto;padding:0 16px}
header.top{border-bottom:1px solid var(--border);background:rgba(10,22,40,.75);backdrop-filter:blur(10px);position:sticky;top:0;z-index:5}
header.top .wrap{display:flex;align-items:center;gap:16px;min-height:60px;flex-wrap:wrap;padding-top:8px;padding-bottom:8px}
.brand{display:flex;align-items:center;gap:10px;color:var(--heading);font-weight:700;font-family:var(--mono);font-size:1.05rem;white-space:nowrap}
.brand .dot{width:10px;height:10px;border-radius:50%;background:var(--accent);box-shadow:0 0 12px var(--accent)}
.brand small{color:var(--dim);font-weight:400}
nav.main{display:flex;gap:4px;flex-wrap:wrap;margin-left:auto}
nav.main a{padding:6px 10px;border-radius:8px;color:var(--text);font-size:.95rem}
nav.main a:hover,nav.main a.active{background:var(--surface2);color:var(--heading);text-decoration:none}
.who{display:flex;align-items:center;gap:8px;color:var(--dim);font-size:.9rem;font-family:var(--mono)}
.avatar{width:26px;height:26px;border-radius:50%;background:linear-gradient(135deg,var(--accent),var(--gold));color:#0a1628;display:inline-flex;align-items:center;justify-content:center;font-weight:700;font-size:.8rem;font-family:var(--mono)}
main{padding:28px 0 64px}
h1{font-size:1.65rem;color:var(--heading);margin:0 0 6px;letter-spacing:-.01em}
h2{font-size:1.15rem;color:var(--heading);margin:32px 0 12px;display:flex;align-items:center;gap:10px}
h2 .count{color:var(--dim);font-weight:400;font-size:.9rem;font-family:var(--mono)}
h3{font-size:1rem;color:var(--heading);margin:0 0 8px}
p.lead{color:var(--dim);margin:0 0 20px;max-width:64ch}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:14px}
.card{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:16px;display:flex;flex-direction:column;gap:10px;min-width:0}
.card.elevated{background:var(--surface2)}
.card .title{display:flex;justify-content:space-between;gap:10px;align-items:flex-start}
.card .title a{color:var(--heading);font-weight:600;font-size:1.05rem}
.card .host{font-family:var(--mono);font-size:.82rem;color:var(--dim);word-break:break-all}
.card .desc{color:var(--text);font-size:.93rem;margin:0}
.card .actions{display:flex;gap:8px;flex-wrap:wrap;margin-top:auto;padding-top:4px}
.badge{display:inline-flex;align-items:center;gap:6px;padding:2px 9px;border-radius:999px;font-size:.75rem;font-family:var(--mono);border:1px solid var(--border);color:var(--dim);white-space:nowrap}
.badge::before{content:"";width:6px;height:6px;border-radius:50%;background:currentColor}
.badge.private{color:var(--gold);border-color:rgba(240,184,96,.4)}
.badge.public_unlisted{color:var(--accent2);border-color:rgba(129,212,250,.4)}
.badge.public_listed{color:var(--ok);border-color:rgba(110,231,168,.4)}
.badge.off{color:var(--bad);border-color:rgba(255,123,138,.4)}
.badge.admin{color:var(--gold)}
.btn{display:inline-flex;align-items:center;gap:6px;padding:8px 14px;border-radius:9px;border:1px solid var(--border);background:var(--surface2);color:var(--heading);font:inherit;font-size:.92rem;cursor:pointer;text-decoration:none;line-height:1.2}
.btn:hover{border-color:var(--accent);text-decoration:none}
.btn.primary{background:var(--accent);color:#0a1628;border-color:var(--accent);font-weight:600}
.btn.primary:hover{background:var(--accent2)}
.btn.danger{color:var(--bad)}.btn.danger:hover{border-color:var(--bad)}
.btn.small{padding:5px 10px;font-size:.82rem}
.btn[disabled]{opacity:.5;cursor:not-allowed}
form.inline{display:inline}
.row{display:flex;gap:10px;flex-wrap:wrap;align-items:center}
.stack{display:flex;flex-direction:column;gap:12px}
.panel{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:18px}
.panel+.panel{margin-top:14px}
.two{display:grid;grid-template-columns:1fr 1fr;gap:14px}
@media (max-width:760px){.two{grid-template-columns:1fr}}
label{display:block;font-size:.85rem;color:var(--dim);margin-bottom:4px}
input[type=text],select{width:100%;padding:9px 11px;border-radius:9px;border:1px solid var(--border);background:var(--bg);color:var(--heading);font:inherit;font-size:.95rem}
input[type=text]:focus,select:focus{outline:none;border-color:var(--accent);box-shadow:0 0 0 3px rgba(79,195,247,.15)}
.field{display:flex;flex-direction:column;gap:2px;min-width:0;flex:1}
.hint{color:var(--dim);font-size:.82rem}
.radios{display:flex;flex-direction:column;gap:8px}
.radios label{display:flex;gap:10px;align-items:flex-start;color:var(--text);font-size:.95rem;cursor:pointer;padding:10px;border:1px solid var(--border);border-radius:9px;margin:0}
.radios label:has(input:checked){border-color:var(--accent);background:rgba(79,195,247,.06)}
.radios input{margin-top:4px}
.radios .hint{display:block}
table{width:100%;border-collapse:collapse;font-size:.92rem}
th{text-align:left;color:var(--dim);font-weight:500;font-size:.8rem;text-transform:uppercase;letter-spacing:.04em;padding:8px 8px;border-bottom:1px solid var(--border)}
td{padding:10px 8px;border-bottom:1px solid var(--border);vertical-align:middle}
tr:last-child td{border-bottom:0}
.table-wrap{overflow-x:auto;-webkit-overflow-scrolling:touch}
.table-wrap td,.table-wrap th{white-space:nowrap}
.flash{padding:12px 14px;border-radius:10px;border:1px solid;margin:0 0 18px;font-size:.95rem}
.flash.ok{border-color:rgba(110,231,168,.4);background:rgba(110,231,168,.08);color:var(--ok)}
.flash.err{border-color:rgba(255,123,138,.4);background:rgba(255,123,138,.08);color:var(--bad)}
.secret{background:var(--bg);border:1px dashed var(--gold);border-radius:10px;padding:14px;word-break:break-all;font-family:var(--mono);font-size:.9rem;color:var(--heading)}
.empty{color:var(--dim);padding:22px;border:1px dashed var(--border);border-radius:var(--radius);text-align:center}
.kv{display:grid;grid-template-columns:max-content 1fr;gap:6px 16px;font-size:.93rem}
.kv dt{color:var(--dim);margin:0}.kv dd{margin:0;word-break:break-word}
.status-page{max-width:560px;margin:8vh auto 0;text-align:center}
.status-page .icon{font-size:2.6rem;margin-bottom:8px}
.status-page p{color:var(--dim)}
footer{border-top:1px solid var(--border);color:var(--dim);font-size:.82rem;padding:18px 0;font-family:var(--mono)}
footer .wrap{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap}
.muted{color:var(--dim)}
.right{margin-left:auto}
@media (max-width:600px){main{padding-top:18px}h1{font-size:1.4rem}.card{padding:14px}header.top .wrap{gap:8px}.who{margin-left:auto}nav.main{order:3;width:100%;margin-left:0}}
"#;
