// repo.box publisher probe. Shows the edge-injected identity, the release it
// runs, a /data counter that survives releases, and a same-origin call to the
// platform AI endpoint. No dependencies, no login of its own.
const http = require('http');
const fs = require('fs');
const path = require('path');

const port = Number(process.env.PORT || 8080);
const dataDir = process.env.REPOBOX_DATA_DIR || '/data';
const page = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');

function bump() {
  const f = path.join(dataDir, 'boots.txt');
  let n = 0;
  try { n = Number(fs.readFileSync(f, 'utf8')) || 0; } catch {}
  try { fs.writeFileSync(f, String(n + 1)); return n + 1; } catch (e) { return `unwritable: ${e.code}`; }
}
const boots = bump();
console.log(`probe ${process.env.APP_VERSION || 'v?'} release ${process.env.REPOBOX_RELEASE} listening on ${port}, boots=${boots}`);

http.createServer((req, res) => {
  const url = new URL(req.url, 'http://x');
  if (url.pathname === '/healthz') {
    res.writeHead(200, { 'content-type': 'text/plain' });
    return res.end('ok\n');
  }
  if (url.pathname === '/whoami.json') {
    const id = {};
    for (const [k, v] of Object.entries(req.headers)) if (k.startsWith('x-repobox-')) id[k] = v;
    res.writeHead(200, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({
      identity: id,
      app: process.env.REPOBOX_APP,
      release: process.env.REPOBOX_RELEASE,
      version: process.env.APP_VERSION || null,
      ai_path: process.env.REPOBOX_AI_CHAT_PATH,
      boots,
      uid: process.getuid(),
    }));
  }
  if (url.pathname === '/') {
    console.log(`page view by user id ${req.headers['x-repobox-user-id'] || '-'}`);
    res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
    return res.end(page.replace('{{VERSION}}', process.env.APP_VERSION || 'v?'));
  }
  res.writeHead(404, { 'content-type': 'text/plain' });
  res.end('not found\n');
}).listen(port, '0.0.0.0');
