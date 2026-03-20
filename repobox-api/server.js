const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const SUBS_FILE = path.join(__dirname, 'subscribers.json');
const LEADERBOARD_FILE = path.join(__dirname, 'leaderboard.json');
const PORT = 3350;
const SECRET = crypto.randomBytes(32).toString('hex'); // rotates on restart — fine
const GAME_TICK_MS = 125;
const usedTokens = new Set();

function readJson(file, fallback) {
  try { return JSON.parse(fs.readFileSync(file, 'utf8')); }
  catch { return fallback; }
}
function writeJson(file, data) {
  fs.writeFileSync(file, JSON.stringify(data, null, 2));
}

function loadSubs() { return readJson(SUBS_FILE, []); }
function saveSubs(subs) { writeJson(SUBS_FILE, subs); }
function loadLeaderboard() { return readJson(LEADERBOARD_FILE, []); }
function saveLeaderboard(rows) { writeJson(LEADERBOARD_FILE, rows); }

function makeToken(ts) {
  return crypto.createHmac('sha256', SECRET).update(String(ts)).digest('hex');
}

function verifyToken(token, ts) {
  if (!token || !ts) return false;
  const expected = makeToken(ts);
  return crypto.timingSafeEqual(Buffer.from(token, 'hex'), Buffer.from(expected, 'hex'));
}

const server = http.createServer((req, res) => {
  res.setHeader('Content-Type', 'application/json');

  // Issue a game token (timestamped, HMAC-signed)
  if (req.method === 'GET' && req.url === '/api/game-token') {
    const ts = Date.now();
    const token = makeToken(ts);
    res.writeHead(200);
    return res.end(JSON.stringify({ token, ts }));
  }

  if (req.method === 'GET' && req.url.startsWith('/api/leaderboard')) {
    const url = new URL(req.url, 'http://localhost');
    const full = url.searchParams.get('full') === '1';
    const rows = loadLeaderboard()
      .sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1));
    res.writeHead(200);
    return res.end(JSON.stringify({ ok: true, rows: full ? rows : rows.slice(0, 20) }));
  }

  if (req.method === 'POST' && req.url === '/api/leaderboard') {
    let body = '';
    req.on('data', c => body += c);
    req.on('end', () => {
      try {
        const { name, score, token, ts } = JSON.parse(body || '{}');
        const cleanName = String(name || '').trim().slice(0, 24);
        const numScore = Number(score);

        if (!cleanName || !Number.isFinite(numScore) || numScore <= 0) {
          res.writeHead(400);
          return res.end(JSON.stringify({ error: 'Invalid name or score' }));
        }

        // Validate game token
        if (!token || !ts) {
          res.writeHead(403);
          return res.end(JSON.stringify({ error: 'Missing game token. Nice try 🐍' }));
        }

        try {
          if (!verifyToken(token, ts)) {
            res.writeHead(403);
            return res.end(JSON.stringify({ error: 'Invalid token. Play the game! 🎮' }));
          }
        } catch {
          res.writeHead(403);
          return res.end(JSON.stringify({ error: 'Bad token format' }));
        }

        // Single-use token
        const tokenKey = token.slice(0, 16);
        if (usedTokens.has(tokenKey)) {
          res.writeHead(403);
          return res.end(JSON.stringify({ error: 'Token already used. One game, one score.' }));
        }

        // Time validation: did enough real time pass?
        const elapsed = Date.now() - Number(ts);
        const minTime = Math.floor(numScore) * GAME_TICK_MS;
        // Allow 500ms grace for network latency
        if (elapsed < minTime - 500) {
          const needed = Math.ceil(minTime / 1000);
          res.writeHead(403);
          return res.end(JSON.stringify({
            error: `Too fast. Score ${Math.floor(numScore)} needs ${needed}s of gameplay. You took ${(elapsed/1000).toFixed(1)}s. 🤔`
          }));
        }

        // Max session 10 minutes (score ~4800 theoretical max)
        if (elapsed > 600_000) {
          res.writeHead(403);
          return res.end(JSON.stringify({ error: 'Token expired. Start a new game.' }));
        }

        // All good — burn the token and save
        usedTokens.add(tokenKey);
        // Cleanup old tokens periodically (keep set small)
        if (usedTokens.size > 10000) {
          const arr = [...usedTokens];
          arr.splice(0, 5000);
          usedTokens.clear();
          arr.forEach(t => usedTokens.add(t));
        }

        const rows = loadLeaderboard();
        rows.push({
          name: cleanName,
          score: Math.floor(numScore),
          ts: new Date().toISOString()
        });

        rows.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1));
        saveLeaderboard(rows.slice(0, 200));

        res.writeHead(200);
        return res.end(JSON.stringify({ ok: true }));
      } catch {
        res.writeHead(400);
        return res.end(JSON.stringify({ error: 'Bad request' }));
      }
    });
    return;
  }

  if (req.method === 'POST' && req.url === '/api/subscribe') {
    let body = '';
    req.on('data', c => body += c);
    req.on('end', () => {
      try {
        const { email } = JSON.parse(body);
        if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
          res.writeHead(400);
          return res.end(JSON.stringify({ error: 'Invalid email' }));
        }
        const subs = loadSubs();
        if (subs.find(s => s.email === email.toLowerCase())) {
          res.writeHead(200);
          return res.end(JSON.stringify({ ok: true, msg: 'Already subscribed' }));
        }
        subs.push({ email: email.toLowerCase(), ts: new Date().toISOString() });
        saveSubs(subs);
        res.writeHead(200);
        res.end(JSON.stringify({ ok: true }));
      } catch {
        res.writeHead(400);
        res.end(JSON.stringify({ error: 'Bad request' }));
      }
    });
    return;
  }

  res.writeHead(404);
  res.end(JSON.stringify({ error: 'Not found' }));
});

server.listen(PORT, '127.0.0.1', () => console.log(`repobox-api on :${PORT}`));
