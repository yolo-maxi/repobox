// Grid Snake Easter Egg — repo.box
// Self-contained: creates its own canvas element.
// Include on any page: <script src="/snake.js"></script>
(function() {
  const canvas = document.createElement('canvas');
  canvas.id = 'snake-canvas';
  Object.assign(canvas.style, {
    position: 'fixed', top: '0', left: '0',
    width: '100%', height: '100%',
    pointerEvents: 'none', zIndex: '50'
  });
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  let W, H;
  function resize() { W = canvas.width = window.innerWidth; H = canvas.height = window.innerHeight; }
  resize();
  window.addEventListener('resize', resize);

  const G = 20, MAX_LEN = 12;
  // Always max 2 snakes
  function snakeCount() { return W >= 900 ? 2 : 1; }

  let snakes = [], lastScroll = window.scrollY, scrollAccum = 0, scrollDir = 1, idleTimer = null;

  // sides: 0=top, 1=right, 2=bottom, 3=left
  function makeSnake(side) {
    const sy = window.scrollY;
    let x, y, dx, dy;
    if (side === 0) { x = Math.round((G*3+Math.random()*(W-G*6))/G)*G; y = sy-G; dx=0; dy=1; }
    else if (side === 1) { x = W+G; y = sy+Math.round((G*3+Math.random()*(H-G*6))/G)*G; dx=-1; dy=0; }
    else if (side === 2) { x = Math.round((G*3+Math.random()*(W-G*6))/G)*G; y = sy+H+G; dx=0; dy=-1; }
    else { x = -G; y = sy+Math.round((G*3+Math.random()*(H-G*6))/G)*G; dx=1; dy=0; }
    const segs = [];
    for (let i = 0; i < 3; i++) segs.push({ x: x-dx*i*G, y: y-dy*i*G });
    return { segs, moveX: dx, moveY: dy, entrySide: side, escaping: false, escapeX: 0, escapeY: 0, escapeTimer: null, dead: false };
  }

  function spawnAll(dir) {
    snakes.forEach(s => { if (s.escapeTimer) clearInterval(s.escapeTimer); });
    snakes = [];
    const count = snakeCount();
    // Pick distinct random sides for each snake
    const sides = [0, 1, 2, 3];
    for (let i = sides.length - 1; i > 0; i--) { const j = Math.floor(Math.random()*(i+1)); [sides[i],sides[j]]=[sides[j],sides[i]]; }
    for (let i = 0; i < count; i++) snakes.push(makeSnake(sides[i]));
    scrollDir = dir;
  }

  function stepOne(s) {
    if (s.dead || !s.segs.length) return;
    const h = s.segs[0]; let nx = h.x, ny = h.y; const sy = window.scrollY;
    if (s.escaping) { nx += s.escapeX*G; ny += s.escapeY*G; }
    else {
      const r = Math.random();
      if (r < 0.5) { nx += s.moveX*G; ny += s.moveY*G; }
      else if (r < 0.75) { nx += s.moveX*G+(s.moveY?-G:0); ny += s.moveY*G+(s.moveX?-G:0); }
      else { nx += s.moveX*G+(s.moveY?G:0); ny += s.moveY*G+(s.moveX?G:0); }
      if (nx < G) nx = G*2; if (nx > W-G) nx = W-G*2;
      if (ny < sy-G*2) ny = sy; if (ny > sy+H+G*2) ny = sy+H;
    }
    s.segs.unshift({ x: nx, y: ny });
    if (s.segs.length > MAX_LEN) s.segs.pop();
    if (s.escaping && s.segs.every(p => p.x<-G||p.x>W+G||p.y<sy-G*2||p.y>sy+H+G*2)) {
      s.dead = true; if (s.escapeTimer) { clearInterval(s.escapeTimer); s.escapeTimer = null; }
    }
  }

  function escapeOne(s) {
    if (s.escaping || s.dead) return;
    s.escaping = true;
    const h = s.segs[0], sy = window.scrollY;
    // Nearest edge EXCEPT the entry side
    const allDirs = [
      {ex:0,ey:-1,d:h.y-sy, side:0},    // top
      {ex:1,ey:0,d:W-h.x, side:1},       // right
      {ex:0,ey:1,d:(sy+H)-h.y, side:2},  // bottom
      {ex:-1,ey:0,d:h.x, side:3}          // left
    ];
    const candidates = allDirs.filter(d => d.side !== s.entrySide).sort((a,b) => a.d - b.d);
    const pick = candidates[0];
    s.escapeX = pick.ex; s.escapeY = pick.ey;
    let steps = 0;
    s.escapeTimer = setInterval(() => {
      stepOne(s); steps++;
      if (steps > 25 || s.dead) { clearInterval(s.escapeTimer); s.escapeTimer = null; s.dead = true; }
    }, 40);
  }

  function escapeAll() { snakes.forEach(escapeOne); }
  function anyAlive() { return snakes.some(s => !s.dead); }
  function resetIdle() { clearTimeout(idleTimer); idleTimer = setTimeout(escapeAll, 800); }

  window.addEventListener('scroll', function() {
    const sy = window.scrollY, delta = sy - lastScroll; lastScroll = sy;
    if (delta === 0) return;
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

  function render() {
    ctx.clearRect(0, 0, W, H);
    const sy = window.scrollY;
    ctx.lineCap = 'round'; ctx.lineJoin = 'round';
    snakes.forEach(snake => {
      if (snake.dead || snake.segs.length < 2) return;
      for (let i = 0; i < snake.segs.length - 1; i++) {
        const t = 1 - i / snake.segs.length;
        ctx.beginPath();
        ctx.strokeStyle = `rgba(79,195,247,${0.25 + 0.75*t})`;
        ctx.lineWidth = 2 + t;
        if (i === 0) { ctx.shadowColor = '#4fc3f7'; ctx.shadowBlur = 6; } else { ctx.shadowBlur = 0; }
        ctx.moveTo(snake.segs[i].x, snake.segs[i].y - sy);
        ctx.lineTo(snake.segs[i+1].x, snake.segs[i+1].y - sy);
        ctx.stroke();
      }
    });
    ctx.shadowBlur = 0;
    requestAnimationFrame(render);
  }
  requestAnimationFrame(render);
})();
