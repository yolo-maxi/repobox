(() => {
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (prefersReducedMotion) return;

  const hero = document.querySelector('.hero-with-mantas');
  const canvas = document.getElementById('hero-manta-canvas');
  if (!hero || !canvas) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  let width = 0;
  let height = 0;
  let dpr = 1;
  let rafId = null;
  let startTs = null;

  const isMobile = window.innerWidth < 900;
  const fleetSize = isMobile ? 3 : 4;
  const sizeRange = isMobile ? [0.24, 0.32] : [0.20, 0.30]; // vw
  const durationMs = isMobile ? 5200 : 4800;

  const mantas = [];

  function resize() {
    const rect = hero.getBoundingClientRect();
    width = Math.max(1, rect.width);
    height = Math.max(1, rect.height);
    dpr = Math.min(window.devicePixelRatio || 1, 2);

    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  function buildFleet() {
    mantas.length = 0;
    const centerY = height * 0.44;

    for (let i = 0; i < fleetSize; i++) {
      const sizeVW = sizeRange[0] + Math.random() * (sizeRange[1] - sizeRange[0]);
      const w = width * sizeVW;
      const h = w * 0.38;

      mantas.push({
        w,
        h,
        startX: -w - i * (width * 0.08 + 30),
        endX: width + w + i * 40,
        y: centerY + (i - (fleetSize - 1) / 2) * (height * 0.08),
        phase: Math.random() * Math.PI * 2,
        flapFreq: 0.9 + Math.random() * 0.4,
        alpha: 0.10 + Math.random() * 0.06,
        delayMs: i * 240,
        driftAmp: 5 + Math.random() * 6,
      });
    }
  }

  function drawManta(m, x, t) {
    const wing = Math.sin((t / 1000) * (Math.PI * 2) * m.flapFreq + m.phase) * (m.h * 0.12);
    const tailSway = Math.sin((t / 1000) * (Math.PI * 2) * (m.flapFreq * 0.6) + m.phase) * (m.h * 0.10);

    ctx.save();
    ctx.translate(x, m.y + Math.sin((t / 1000) * 0.9 + m.phase) * m.driftAmp);

    ctx.beginPath();
    ctx.moveTo(-m.w * 0.5, 0);
    ctx.quadraticCurveTo(-m.w * 0.24, -m.h * 0.65 - wing, 0, -m.h * 0.2);
    ctx.quadraticCurveTo(m.w * 0.24, -m.h * 0.65 - wing, m.w * 0.5, 0);
    ctx.quadraticCurveTo(m.w * 0.24, m.h * 0.65 + wing, 0, m.h * 0.2);
    ctx.quadraticCurveTo(-m.w * 0.24, m.h * 0.65 + wing, -m.w * 0.5, 0);
    ctx.closePath();

    const grad = ctx.createLinearGradient(-m.w * 0.55, 0, m.w * 0.55, 0);
    grad.addColorStop(0, `rgba(79,195,247,${m.alpha * 0.65})`);
    grad.addColorStop(0.5, `rgba(129,212,250,${m.alpha})`);
    grad.addColorStop(1, `rgba(79,195,247,${m.alpha * 0.65})`);

    ctx.fillStyle = grad;
    ctx.fill();

    // Tail
    ctx.beginPath();
    ctx.moveTo(-m.w * 0.5, m.h * 0.02);
    ctx.quadraticCurveTo(-m.w * 0.68, m.h * 0.18 + tailSway, -m.w * 0.86, m.h * 0.04 + tailSway);
    ctx.strokeStyle = `rgba(129,212,250,${m.alpha * 0.8})`;
    ctx.lineWidth = Math.max(1.5, m.w * 0.008);
    ctx.stroke();

    ctx.restore();
  }

  function animate(ts) {
    if (startTs == null) startTs = ts;
    const elapsed = ts - startTs;

    ctx.clearRect(0, 0, width, height);

    let allDone = true;

    for (const m of mantas) {
      const t = elapsed - m.delayMs;
      if (t < 0) {
        allDone = false;
        continue;
      }

      const p = Math.min(1, t / durationMs);
      const eased = 1 - Math.pow(1 - p, 3);
      const x = m.startX + (m.endX - m.startX) * eased;

      if (p < 1) allDone = false;
      drawManta(m, x, ts);
    }

    if (!allDone) {
      rafId = requestAnimationFrame(animate);
      return;
    }

    // Fade out and stop after one pass.
    canvas.style.transition = 'opacity 700ms ease';
    canvas.style.opacity = '0';
    setTimeout(() => {
      ctx.clearRect(0, 0, width, height);
    }, 740);
  }

  function runOnce() {
    resize();
    buildFleet();
    canvas.classList.add('active');
    canvas.style.opacity = '1';
    startTs = null;
    if (rafId) cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(animate);
  }

  window.addEventListener('load', () => {
    setTimeout(runOnce, 700);
  }, { once: true });

  // Keep proportions correct if user rotates/resizes before the one-shot starts.
  window.addEventListener('resize', resize, { passive: true });
})();
