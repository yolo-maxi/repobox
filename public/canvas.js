// Unified canvas — repo.box
// Grid background + cursor glow + scroll-driven snakes + interactive snake game
// Include on any page: <script src="/canvas.js"></script>
(function () {
  const G = 20, MAJ = 80, MAX_LEN = 12;
  const canvas = document.createElement('canvas');
  Object.assign(canvas.style, {
    position: 'fixed', top: '0', left: '0',
    width: '100vw', height: '100vh',
    pointerEvents: 'none', zIndex: '0'
  });
  document.body.prepend(canvas);
  const ctx = canvas.getContext('2d');
  let W, H, dpr = 1;

  function resize() {
    dpr = window.devicePixelRatio || 1;
    W = window.innerWidth;
    H = window.innerHeight;
    canvas.width = W * dpr;
    canvas.height = H * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }
  resize();
  window.addEventListener('resize', resize);

  // --- Mouse tracking ---
  let mouseX = -9999, mouseY = -9999, hasMouse = false;
  const isMobile = 'ontouchstart' in window || navigator.maxTouchPoints > 0;
  if (!isMobile) {
    window.addEventListener('mousemove', function (e) {
      mouseX = e.clientX; mouseY = e.clientY; hasMouse = true;
    });
    window.addEventListener('mouseleave', function () { hasMouse = false; });
  }

  // --- Grid drawing ---
  function drawGrid() {
    const sy = window.scrollY;
    ctx.fillStyle = '#0a1628';
    ctx.fillRect(0, 0, W, H);

    const offsetY = -(sy % G);
    const offsetYMaj = -(sy % MAJ);

    ctx.beginPath();
    ctx.strokeStyle = 'rgba(50,100,160,0.08)';
    ctx.lineWidth = 1;
    for (let x = 0; x <= W; x += G) { ctx.moveTo(x + 0.5, 0); ctx.lineTo(x + 0.5, H); }
    for (let y = offsetY; y <= H; y += G) { ctx.moveTo(0, y + 0.5); ctx.lineTo(W, y + 0.5); }
    ctx.stroke();

    ctx.beginPath();
    ctx.strokeStyle = 'rgba(50,100,160,0.18)';
    ctx.lineWidth = 1;
    for (let x = 0; x <= W; x += MAJ) { ctx.moveTo(x + 0.5, 0); ctx.lineTo(x + 0.5, H); }
    for (let y = offsetYMaj; y <= H; y += MAJ) { ctx.moveTo(0, y + 0.5); ctx.lineTo(W, y + 0.5); }
    ctx.stroke();
  }

  // --- Cursor glow ---
  function drawGlow() {
    if (!hasMouse || isMobile) return;
    const sy = window.scrollY;
    const R = 100, R2 = R * R;
    const startX = Math.floor((mouseX - R) / G) * G;
    const endX = Math.ceil((mouseX + R) / G) * G;
    const worldY = mouseY + sy;
    const startY = Math.floor((worldY - R) / G) * G;
    const endY = Math.ceil((worldY + R) / G) * G;

    for (let gx = startX; gx <= endX; gx += G) {
      for (let gy = startY; gy <= endY; gy += G) {
        const screenY = gy - sy;
        const dx = gx - mouseX, dy = screenY - mouseY;
        const d2 = dx * dx + dy * dy;
        if (d2 > R2) continue;
        const alpha = 0.5 * (1 - Math.sqrt(d2) / R);
        ctx.beginPath();
        ctx.fillStyle = `rgba(79,195,247,${alpha})`;
        ctx.arc(gx, screenY, 2, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }

  // =========================================================================
  // PASSIVE SNAKES
  // =========================================================================
  function snakeCount() { return W >= 900 ? 2 : 1; }
  let snakes = [], lastScroll = window.scrollY, scrollAccum = 0, scrollDir = 1, idleTimer = null;

  function makeSnake(side) {
    const sy = window.scrollY;
    let x, y, dx, dy;
    if (side === 0) { x = Math.round((G * 3 + Math.random() * (W - G * 6)) / G) * G; y = sy - G; dx = 0; dy = 1; }
    else if (side === 1) { x = W + G; y = sy + Math.round((G * 3 + Math.random() * (H - G * 6)) / G) * G; dx = -1; dy = 0; }
    else if (side === 2) { x = Math.round((G * 3 + Math.random() * (W - G * 6)) / G) * G; y = sy + H + G; dx = 0; dy = -1; }
    else { x = -G; y = sy + Math.round((G * 3 + Math.random() * (H - G * 6)) / G) * G; dx = 1; dy = 0; }
    const segs = [];
    for (let i = 0; i < 3; i++) segs.push({ x: x - dx * i * G, y: y - dy * i * G });
    return { segs, moveX: dx, moveY: dy, entrySide: side, escaping: false, escapeX: 0, escapeY: 0, escapeTimer: null, dead: false };
  }

  function spawnAll(dir) {
    snakes.forEach(s => { if (s.escapeTimer) clearInterval(s.escapeTimer); });
    snakes = [];
    const count = snakeCount();
    const sides = [0, 1, 2, 3];
    for (let i = sides.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [sides[i], sides[j]] = [sides[j], sides[i]]; }
    for (let i = 0; i < count; i++) snakes.push(makeSnake(sides[i]));
    scrollDir = dir;
  }

  function stepOne(s) {
    if (s.dead || !s.segs.length) return;
    const h = s.segs[0]; let nx = h.x, ny = h.y; const sy = window.scrollY;
    if (s.escaping) { nx += s.escapeX * G; ny += s.escapeY * G; }
    else {
      const r = Math.random();
      if (r < 0.5) { nx += s.moveX * G; ny += s.moveY * G; }
      else if (r < 0.75) { nx += s.moveX * G + (s.moveY ? -G : 0); ny += s.moveY * G + (s.moveX ? -G : 0); }
      else { nx += s.moveX * G + (s.moveY ? G : 0); ny += s.moveY * G + (s.moveX ? G : 0); }
      if (nx < G) nx = G * 2; if (nx > W - G) nx = W - G * 2;
      if (ny < sy - G * 2) ny = sy; if (ny > sy + H + G * 2) ny = sy + H;
    }
    s.segs.unshift({ x: nx, y: ny });
    if (s.segs.length > MAX_LEN) s.segs.pop();
    if (s.escaping && s.segs.every(p => p.x < -G || p.x > W + G || p.y < sy - G * 2 || p.y > sy + H + G * 2)) {
      s.dead = true; if (s.escapeTimer) { clearInterval(s.escapeTimer); s.escapeTimer = null; }
    }
  }

  function escapeOne(s) {
    if (s.escaping || s.dead) return;
    s.escaping = true;
    const h = s.segs[0], sy = window.scrollY;
    const allDirs = [
      { ex: 0, ey: -1, d: h.y - sy, side: 0 },
      { ex: 1, ey: 0, d: W - h.x, side: 1 },
      { ex: 0, ey: 1, d: (sy + H) - h.y, side: 2 },
      { ex: -1, ey: 0, d: h.x, side: 3 }
    ];
    const pick = allDirs.filter(d => d.side !== s.entrySide).sort((a, b) => a.d - b.d)[0];
    s.escapeX = pick.ex; s.escapeY = pick.ey;
    let steps = 0;
    s.escapeTimer = setInterval(() => {
      stepOne(s); steps++;
      if (steps > 25 || s.dead) { clearInterval(s.escapeTimer); s.escapeTimer = null; s.dead = true; }
    }, 40);
  }

  function escapeAll() { snakes.forEach(escapeOne); }
  function anyAlive() { return snakes.some(s => !s.dead); }

  // Coast: after scroll stops, snakes keep moving on their own for 700ms
  let coastInterval = null;
  function startCoast() {
    stopCoast();
    coastInterval = setInterval(() => {
      snakes.forEach(s => { if (!s.escaping && !s.dead) stepOne(s); });
    }, 80);
  }
  function stopCoast() { if (coastInterval) { clearInterval(coastInterval); coastInterval = null; } }

  function resetIdle() {
    clearTimeout(idleTimer);
    startCoast();
    // After 700ms of coasting, stop and pause briefly (400ms) before escaping
    idleTimer = setTimeout(() => {
      stopCoast();
      setTimeout(escapeAll, 400);
    }, 700);
  }

  window.addEventListener('scroll', function () {
    if (gameState !== 'idle') return;
    const sy = window.scrollY, delta = sy - lastScroll; lastScroll = sy;
    if (delta === 0) return;
    stopCoast();
    const dir = delta > 0 ? 1 : -1;
    if (!anyAlive()) { spawnAll(dir); scrollAccum = 0; }
    if (dir !== scrollDir && anyAlive()) {
      escapeAll();
      setTimeout(() => { if (!anyAlive()) spawnAll(dir); }, 300);
      return;
    }
    scrollAccum += Math.abs(delta);
    while (scrollAccum >= G) { scrollAccum -= G; snakes.forEach(s => { if (!s.escaping && !s.dead) stepOne(s); }); }
    resetIdle();
  }, { passive: true });

  function drawSnakes() {
    const sy = window.scrollY;
    ctx.lineCap = 'round'; ctx.lineJoin = 'round';
    snakes.forEach(snake => {
      if (snake.dead || snake.segs.length < 2) return;
      for (let i = 0; i < snake.segs.length - 1; i++) {
        const t = 1 - i / snake.segs.length;
        ctx.beginPath();
        ctx.strokeStyle = `rgba(79,195,247,${0.25 + 0.75 * t})`;
        ctx.lineWidth = 2 + t;
        if (i === 0) { ctx.shadowColor = '#4fc3f7'; ctx.shadowBlur = 6; } else { ctx.shadowBlur = 0; }
        ctx.moveTo(snake.segs[i].x, snake.segs[i].y - sy);
        ctx.lineTo(snake.segs[i + 1].x, snake.segs[i + 1].y - sy);
        ctx.stroke();
      }
    });
    ctx.shadowBlur = 0;
  }

  function hitSnakeAt(screenX, screenY) {
    const worldY = screenY + window.scrollY;
    const hitRadius = 14;
    const hitR2 = hitRadius * hitRadius;
    for (const s of snakes) {
      if (s.dead || !s.segs?.length) continue;
      for (const seg of s.segs) {
        const dx = seg.x - screenX;
        const dy = seg.y - worldY;
        if (dx * dx + dy * dy <= hitR2) return true;
      }
    }
    return false;
  }

  // =========================================================================
  // INTERACTIVE SNAKE GAME
  // =========================================================================
  // States: 'idle' → 'menu' → 'playing' → 'dead' → 'menu' or 'idle'
  let gameState = 'idle';
  let player = null;
  let food = null;
  let score = 0;
  let gameOverTime = 0;
  let gameInterval = null;
  let menuStartTime = 0;
  let gameToken = null;
  let gameTokenTs = null;
  const GAME_SPEED = 125;
  const contentEl = document.querySelector('[style*="z-index:1"]') || document.querySelector('.max-w-\\[720px\\]');

  const DIR_MAP = {
    ArrowUp: [0, -1], ArrowDown: [0, 1], ArrowLeft: [-1, 0], ArrowRight: [1, 0],
    w: [0, -1], W: [0, -1], s: [0, 1], S: [0, 1],
    a: [-1, 0], A: [-1, 0], d: [1, 0], D: [1, 0]
  };
  const ACTIVATE_KEYS = new Set(Object.keys(DIR_MAP));

  // --- HTML overlay for buttons ---
  const overlay = document.createElement('div');
  Object.assign(overlay.style, {
    position: 'fixed', inset: '0', zIndex: '50',
    display: 'none', alignItems: 'center', justifyContent: 'center',
    flexDirection: 'column', gap: '20px', pointerEvents: 'none'
  });
  document.body.appendChild(overlay);

  function makeBtn(text) {
    const btn = document.createElement('button');
    btn.textContent = text;
    Object.assign(btn.style, {
      fontFamily: '"JetBrains Mono", monospace',
      fontSize: '28px', fontWeight: 'bold',
      color: '#4fc3f7', background: 'transparent',
      border: '1px solid rgba(79,195,247,0.3)',
      borderRadius: '8px', padding: '16px 48px',
      cursor: 'crosshair', pointerEvents: 'auto',
      transition: 'border-color 0.2s, text-shadow 0.2s',
      textShadow: '0 0 8px rgba(79,195,247,0.4)',
      outline: 'none'
    });
    btn.addEventListener('mouseenter', () => {
      btn.style.borderColor = 'rgba(79,195,247,0.7)';
      btn.style.textShadow = '0 0 16px rgba(79,195,247,0.8)';
    });
    btn.addEventListener('mouseleave', () => {
      btn.style.borderColor = 'rgba(79,195,247,0.3)';
      btn.style.textShadow = '0 0 8px rgba(79,195,247,0.4)';
    });
    return btn;
  }

  const btnStart = makeBtn('START GAME');
  const btnExit = makeBtn('EXIT');
  const btnPlayAgain = makeBtn('PLAY AGAIN');
  const btnExitDead = makeBtn('EXIT');
  const scoreLabel = document.createElement('div');

  function lockButtons(ms = 2000) {
    const btns = [btnStart, btnExit, btnPlayAgain, btnExitDead];
    btns.forEach(btn => {
      btn.disabled = true;
      btn.style.opacity = '0.45';
      btn.style.pointerEvents = 'none';
    });
    setTimeout(() => {
      btns.forEach(btn => {
        btn.disabled = false;
        btn.style.opacity = '1';
        btn.style.pointerEvents = 'auto';
      });
    }, ms);
  }

  async function submitScoreIfNeeded() {
    if (score <= 0) return;
    const name = (window.prompt('New score. Enter your name for leaderboard:', '') || '').trim();
    if (!name) return;
    try {
      const clean = name.slice(0, 24);
      await fetch('/api/leaderboard', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: clean, score, token: gameToken, ts: gameTokenTs })
      });

      const res = await fetch('/api/leaderboard?full=1');
      const data = await res.json();
      const rows = Array.isArray(data?.rows) ? data.rows : [];
      const top5 = rows.slice(0, 5);

      // Find this exact submission by latest matching name+score
      let pos = -1;
      for (let i = rows.length - 1; i >= 0; i--) {
        if (rows[i].name === clean && rows[i].score === score) {
          pos = i + 1;
          break;
        }
      }
      if (pos === -1) {
        pos = rows.findIndex(r => r.name === clean && r.score === score) + 1;
      }

      const lines = ['TOP 5'];
      top5.forEach((r, i) => lines.push(`${i + 1}. ${r.name} — ${r.score}`));
      if (pos > 0) lines.push('', `YOU: ${clean} — ${score} (position ${pos})`);
      window.alert(lines.join('\n'));
    } catch (_) {}
  }
  Object.assign(scoreLabel.style, {
    fontFamily: '"JetBrains Mono", monospace',
    fontSize: '16px', color: 'rgba(79,195,247,0.6)',
    textAlign: 'center'
  });

  function showMenu() {
    gameState = 'menu';
    menuStartTime = performance.now();
    overlay.innerHTML = '';
    overlay.appendChild(btnStart);
    overlay.appendChild(btnExit);
    overlay.style.display = 'flex';
    lockButtons(2000);
    document.body.style.overflow = 'hidden';
    window.scrollTo(0, 0);
  }

  function showDeadScreen() {
    overlay.innerHTML = '';
    const goText = document.createElement('div');
    goText.textContent = 'GAME OVER';
    Object.assign(goText.style, {
      fontFamily: '"JetBrains Mono", monospace',
      fontSize: '36px', fontWeight: 'bold',
      color: '#4fc3f7',
      textShadow: '0 0 16px rgba(79,195,247,0.6)',
      marginBottom: '4px'
    });
    scoreLabel.textContent = 'SCORE ' + score;
    overlay.appendChild(goText);
    overlay.appendChild(scoreLabel);
    overlay.appendChild(btnPlayAgain);
    overlay.appendChild(btnExitDead);
    overlay.style.display = 'flex';
    lockButtons(2000);
  }

  function hideOverlay() {
    overlay.style.display = 'none';
    overlay.innerHTML = '';
  }

  function setContentFlash(on) {
    if (!contentEl) return;
    if (on) {
      contentEl.style.transition = 'opacity 0.5s ease-in-out';
      contentEl.style.animation = 'content-flash 2.4s ease-in-out infinite';
    } else {
      contentEl.style.animation = '';
      contentEl.style.opacity = '1';
    }
  }

  // Inject flash animation CSS
  const flashStyle = document.createElement('style');
  flashStyle.textContent = `
    @keyframes content-flash {
      0%, 100% { opacity: 0.3; }
      50% { opacity: 0.1; }
    }
  `;
  document.head.appendChild(flashStyle);

  function hideContent() {
    if (!contentEl) return;
    contentEl.style.animation = '';
    contentEl.style.transition = 'opacity 0.3s ease';
    contentEl.style.opacity = '0';
    contentEl.style.pointerEvents = 'none';
  }

  function showContent() {
    if (!contentEl) return;
    contentEl.style.animation = '';
    contentEl.style.transition = 'opacity 0.3s ease';
    contentEl.style.opacity = '1';
    contentEl.style.pointerEvents = '';
  }

  function spawnFood() {
    const cols = Math.floor(W / G) - 2, rows = Math.floor(H / G) - 2;
    let fx, fy, tries = 0;
    do {
      fx = (1 + Math.floor(Math.random() * cols)) * G;
      fy = (1 + Math.floor(Math.random() * rows)) * G;
      tries++;
    } while (tries < 100 && player && player.segs.some(s => s.x === fx && s.y === fy));
    food = { x: fx, y: fy };
  }

  async function startPlaying() {
    gameState = 'playing';
    score = 0;
    hideOverlay();
    hideContent();
    escapeAll();

    // Fetch game token for leaderboard anti-cheat
    try {
      const r = await fetch('/api/game-token');
      const d = await r.json();
      gameToken = d.token; gameTokenTs = d.ts;
    } catch { gameToken = null; gameTokenTs = null; }

    const cx = Math.round(W / 2 / G) * G;
    const cy = Math.round(H / 2 / G) * G;
    const segs = [];
    for (let i = 0; i < 3; i++) segs.push({ x: cx - i * G, y: cy });
    player = { segs, dx: 1, dy: 0, nextDx: 1, nextDy: 0 };

    spawnFood();
    if (gameInterval) clearInterval(gameInterval);
    gameInterval = setInterval(gameTick, GAME_SPEED);
  }

  function die() {
    gameState = 'dead';
    gameOverTime = performance.now();
    if (gameInterval) { clearInterval(gameInterval); gameInterval = null; }
    submitScoreIfNeeded();
    // Show dead screen after brief pause
    setTimeout(showDeadScreen, 800);
  }

  function exitGame() {
    gameState = 'idle';
    player = null;
    food = null;
    if (gameInterval) { clearInterval(gameInterval); gameInterval = null; }
    hideOverlay();
    showContent();
    document.body.style.overflow = '';
  }

  function gameTick() {
    if (!player || gameState !== 'playing') return;

    player.dx = player.nextDx;
    player.dy = player.nextDy;

    const head = player.segs[0];
    let nx = head.x + player.dx * G;
    let ny = head.y + player.dy * G;

    // Edge collision = death (no wrapping)
    if (nx < 0 || nx > Math.floor(W / G) * G || ny < 0 || ny > Math.floor(H / G) * G) {
      die();
      return;
    }

    // Self-collision
    for (let i = 1; i < player.segs.length; i++) {
      if (player.segs[i].x === nx && player.segs[i].y === ny) {
        die();
        return;
      }
    }

    player.segs.unshift({ x: nx, y: ny });

    if (food && nx === food.x && ny === food.y) {
      score++;
      spawnFood();
    } else {
      player.segs.pop();
    }
  }

  // --- Button handlers ---
  btnStart.addEventListener('click', function () { startPlaying(); });
  btnExit.addEventListener('click', function () { exitGame(); });
  btnPlayAgain.addEventListener('click', function () { startPlaying(); });
  btnExitDead.addEventListener('click', function () { exitGame(); });

  // --- Keyboard input ---
  window.addEventListener('keydown', function (e) {
    // Trigger menu from idle
    if (gameState === 'idle' && ACTIVATE_KEYS.has(e.key)) {
      showMenu();
      setContentFlash(true);
      e.preventDefault();
      return;
    }

    // In menu: Enter starts, Escape exits
    if (gameState === 'menu') {
      if (e.key === 'Enter') { setContentFlash(false); startPlaying(); e.preventDefault(); }
      else if (e.key === 'Escape') { setContentFlash(false); exitGame(); e.preventDefault(); }
      return;
    }

    // Playing: steer with arrow/WASD
    if (gameState === 'playing') {
      const dir = DIR_MAP[e.key];
      if (dir && player) {
        if (dir[0] !== -player.dx || dir[1] !== -player.dy) {
          player.nextDx = dir[0];
          player.nextDy = dir[1];
        }
        e.preventDefault();
      }
    }

    // Dead: Enter = play again, Escape = exit
    if (gameState === 'dead') {
      if (e.key === 'Enter') { startPlaying(); e.preventDefault(); }
      else if (e.key === 'Escape') { exitGame(); e.preventDefault(); }
    }
  });

  // Prevent scroll during game
  window.addEventListener('keydown', function (e) {
    if (gameState !== 'idle' && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', ' '].includes(e.key)) {
      e.preventDefault();
    }
  }, { passive: false });

  // --- Mobile: double-tap to trigger menu, swipe to steer ---
  let touchStartX = 0, touchStartY = 0;

  // Trigger menu by tapping/clicking an ambient snake (idle only)
  window.addEventListener('pointerdown', function (e) {
    if (gameState !== 'idle') return;
    if (hitSnakeAt(e.clientX, e.clientY)) {
      showMenu();
      setContentFlash(true);
      e.preventDefault();
    }
  }, { passive: false });

  if (isMobile) {
    window.addEventListener('touchstart', function (e) {
      touchStartX = e.touches[0].clientX;
      touchStartY = e.touches[0].clientY;
    }, { passive: true });

    window.addEventListener('touchend', function (e) {
      if (gameState !== 'playing' || !player) return;
      const dx = e.changedTouches[0].clientX - touchStartX;
      const dy = e.changedTouches[0].clientY - touchStartY;
      if (Math.abs(dx) < 20 && Math.abs(dy) < 20) return;
      let ndx, ndy;
      if (Math.abs(dx) > Math.abs(dy)) { ndx = dx > 0 ? 1 : -1; ndy = 0; }
      else { ndx = 0; ndy = dy > 0 ? 1 : -1; }
      if (ndx !== -player.dx || ndy !== -player.dy) {
        player.nextDx = ndx; player.nextDy = ndy;
      }
    }, { passive: true });

    window.addEventListener('touchmove', function (e) {
      if (gameState !== 'idle') e.preventDefault();
    }, { passive: false });
  }

  // --- Game drawing ---
  function drawFood(now) {
    if (!food) return;
    const pulse = 0.6 + 0.4 * Math.sin(now * 0.006);
    const ringPulse = 0.3 + 0.3 * Math.sin(now * 0.004);
    const ringSize = 8 + 4 * Math.sin(now * 0.003);

    ctx.beginPath();
    ctx.strokeStyle = `rgba(79,195,247,${ringPulse})`;
    ctx.lineWidth = 1;
    ctx.arc(food.x, food.y, ringSize, 0, Math.PI * 2);
    ctx.stroke();

    ctx.beginPath();
    ctx.fillStyle = `rgba(79,195,247,${pulse})`;
    ctx.shadowColor = '#4fc3f7';
    ctx.shadowBlur = 8;
    ctx.arc(food.x, food.y, 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.shadowBlur = 0;
  }

  function drawPlayer() {
    if (!player || player.segs.length < 2) return;
    ctx.lineCap = 'round'; ctx.lineJoin = 'round';
    for (let i = 0; i < player.segs.length - 1; i++) {
      const t = 1 - i / player.segs.length;
      ctx.beginPath();
      ctx.strokeStyle = `rgba(79,195,247,${0.35 + 0.65 * t})`;
      ctx.lineWidth = 3 + t;
      if (i === 0) { ctx.shadowColor = '#4fc3f7'; ctx.shadowBlur = 10; } else { ctx.shadowBlur = 0; }
      ctx.moveTo(player.segs[i].x, player.segs[i].y);
      ctx.lineTo(player.segs[i + 1].x, player.segs[i + 1].y);
      ctx.stroke();
    }
    ctx.shadowBlur = 0;
  }

  function drawScore() {
    if (gameState !== 'playing') return;
    ctx.font = '12px "JetBrains Mono", monospace';
    ctx.fillStyle = 'rgba(79,195,247,0.6)';
    ctx.textAlign = 'right';
    ctx.fillText('SCORE ' + score, W - 20, 32);
    ctx.textAlign = 'left';
  }

  // --- Render loop ---
  function render() {
    const now = performance.now();
    drawGrid();
    drawGlow();

    if (gameState === 'playing' || gameState === 'dead') {
      drawFood(now);
      drawPlayer();
      drawScore();
    } else {
      drawSnakes();
    }

    requestAnimationFrame(render);
  }
  requestAnimationFrame(render);
})();
