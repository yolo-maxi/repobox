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
  let rafId = null;
  let lastTs = 0;
  let running = false;

  const isLikelyLowPower = (navigator.hardwareConcurrency && navigator.hardwareConcurrency <= 4) && window.innerWidth < 900;
  if (isLikelyLowPower) return;

  const mantaCount = window.innerWidth < 900 ? 2 : 4;
  const mantas = Array.from({ length: mantaCount }, (_, i) => {
    const leftToRight = Math.random() > 0.35;
    const baseY = height * (0.22 + (i / Math.max(1, mantaCount - 1)) * 0.56);
    return {
      dir: leftToRight ? 1 : -1,
      x: leftToRight ? -120 - Math.random() * 300 : width + 120 + Math.random() * 300,
      y: baseY + (Math.random() * 40 - 20),
      speed: 18 + Math.random() * 18,
      scale: 0.45 + Math.random() * 0.5,
      driftFreq: 0.00035 + Math.random() * 0.00045,
      flapFreq: 0.0013 + Math.random() * 0.0012,
      phase: Math.random() * Math.PI * 2,
      alpha: 0.06 + Math.random() * 0.07,
      respawnGap: 200 + Math.random() * 500,
    };
  });

  function resize() {
    const rect = hero.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    width = Math.max(1, rect.width);
    height = Math.max(1, rect.height);
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  function drawManta(m, t) {
    const flap = Math.sin(t * m.flapFreq + m.phase);
    const wingLift = 9 * flap;
    const tailSway = Math.sin(t * m.flapFreq * 0.7 + m.phase) * 7;

    ctx.save();
    ctx.translate(m.x, m.y);
    ctx.scale(m.dir * m.scale, m.scale);

    ctx.beginPath();
    ctx.moveTo(-70, 0);
    ctx.quadraticCurveTo(-34, -24 - wingLift, 0, -8);
    ctx.quadraticCurveTo(34, -24 - wingLift, 70, 0);
    ctx.quadraticCurveTo(34, 24 + wingLift, 0, 8);
    ctx.quadraticCurveTo(-34, 24 + wingLift, -70, 0);
    ctx.closePath();

    const grad = ctx.createLinearGradient(-80, 0, 80, 0);
    grad.addColorStop(0, `rgba(79,195,247,${m.alpha * 0.65})`);
    grad.addColorStop(0.5, `rgba(129,212,250,${m.alpha})`);
    grad.addColorStop(1, `rgba(79,195,247,${m.alpha * 0.65})`);
    ctx.fillStyle = grad;
    ctx.fill();

    ctx.beginPath();
    ctx.moveTo(-70, 1);
    ctx.quadraticCurveTo(-100, 10 + tailSway, -126, 2 + tailSway * 0.8);
    ctx.strokeStyle = `rgba(129,212,250,${m.alpha * 0.75})`;
    ctx.lineWidth = 2;
    ctx.stroke();

    ctx.restore();
  }

  function updateManta(m, dt, t) {
    m.x += m.speed * m.dir * dt;
    m.y += Math.sin(t * m.driftFreq + m.phase) * 0.22;

    if (m.dir === 1 && m.x > width + m.respawnGap) {
      m.x = -140 - Math.random() * 280;
      m.y = height * (0.18 + Math.random() * 0.64);
    } else if (m.dir === -1 && m.x < -m.respawnGap) {
      m.x = width + 140 + Math.random() * 280;
      m.y = height * (0.18 + Math.random() * 0.64);
    }
  }

  function frame(ts) {
    if (!running) return;
    if (!lastTs) lastTs = ts;
    const dt = Math.min(0.033, (ts - lastTs) / 1000);
    lastTs = ts;

    ctx.clearRect(0, 0, width, height);

    for (const m of mantas) {
      updateManta(m, dt, ts);
      drawManta(m, ts);
    }

    rafId = requestAnimationFrame(frame);
  }

  function start() {
    if (running) return;
    running = true;
    lastTs = 0;
    canvas.classList.add('active');
    rafId = requestAnimationFrame(frame);
  }

  function stop() {
    running = false;
    if (rafId) cancelAnimationFrame(rafId);
    rafId = null;
    canvas.classList.remove('active');
    ctx.clearRect(0, 0, width, height);
  }

  resize();
  window.addEventListener('resize', resize, { passive: true });

  document.addEventListener('visibilitychange', () => {
    if (document.hidden) stop();
    else start();
  });

  window.addEventListener('load', () => {
    setTimeout(start, 900);
  }, { once: true });
})();
