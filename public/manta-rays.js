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
  let hasRun = false;

  const isMobile = window.innerWidth < 900;
  const fleetSize = isMobile ? 3 : 4;
  const sizeRangeVW = [0.20, 0.30]; // requested: about 20/30vw
  const baseDurationMs = isMobile ? 9200 : 9800; // majestic + slow

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

    // Slightly above center so they cross behind logotype/text elegantly.
    const centerY = height * (isMobile ? 0.45 : 0.43);

    for (let i = 0; i < fleetSize; i++) {
      const depth = i / Math.max(1, fleetSize - 1); // front/back variation
      const sizeVW = sizeRangeVW[0] + Math.random() * (sizeRangeVW[1] - sizeRangeVW[0]);
      const w = width * sizeVW * (1 - depth * 0.08);
      const h = w * 0.38;

      mantas.push({
        w,
        h,
        startX: -w - i * (width * 0.09),
        endX: width + w + i * (width * 0.03),
        y: centerY + (i - (fleetSize - 1) / 2) * (height * 0.07),
        phase: Math.random() * Math.PI * 2,
        flapFreq: 0.34 + Math.random() * 0.16, // very slow wing beats
        alpha: 0.20 - depth * 0.07,
        glow: 0.18 - depth * 0.06,
        blur: depth * 0.7,
        delayMs: i * 620,
        durationMs: baseDurationMs + i * 420,
        driftAmp: 4 + Math.random() * 5,
      });
    }
  }

  function drawManta(m, x, ts) {
    const t = ts / 1000;
    const flap = Math.sin(t * (Math.PI * 2) * m.flapFreq + m.phase);
    const wingLift = flap * (m.h * 0.12);
    const bodyRoll = Math.sin(t * 0.7 + m.phase) * 0.035;
    const tailSway = Math.sin(t * (Math.PI * 2) * (m.flapFreq * 0.7) + m.phase) * (m.h * 0.15);

    const y = m.y + Math.sin(t * 0.85 + m.phase) * m.driftAmp;

    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(bodyRoll);

    if (m.blur > 0.01) {
      ctx.filter = `blur(${m.blur}px)`;
    }

    // Soft aura for epic, underwater feel.
    ctx.beginPath();
    ctx.ellipse(0, 0, m.w * 0.52, m.h * 0.48, 0, 0, Math.PI * 2);
    const aura = ctx.createRadialGradient(0, 0, m.h * 0.08, 0, 0, m.w * 0.62);
    aura.addColorStop(0, `rgba(129, 212, 250, ${m.glow})`);
    aura.addColorStop(1, 'rgba(79, 195, 247, 0)');
    ctx.fillStyle = aura;
    ctx.fill();

    // Main wings + body silhouette.
    ctx.beginPath();
    ctx.moveTo(-m.w * 0.50, 0);
    ctx.bezierCurveTo(-m.w * 0.34, -m.h * 0.70 - wingLift, -m.w * 0.13, -m.h * 0.44, 0, -m.h * 0.17);
    ctx.bezierCurveTo(m.w * 0.13, -m.h * 0.44, m.w * 0.34, -m.h * 0.70 - wingLift, m.w * 0.50, 0);
    ctx.bezierCurveTo(m.w * 0.34, m.h * 0.70 + wingLift, m.w * 0.13, m.h * 0.44, 0, m.h * 0.17);
    ctx.bezierCurveTo(-m.w * 0.13, m.h * 0.44, -m.w * 0.34, m.h * 0.70 + wingLift, -m.w * 0.50, 0);
    ctx.closePath();

    const wingGrad = ctx.createLinearGradient(-m.w * 0.56, 0, m.w * 0.56, 0);
    wingGrad.addColorStop(0, `rgba(36, 96, 140, ${m.alpha * 0.95})`);
    wingGrad.addColorStop(0.5, `rgba(108, 190, 234, ${m.alpha})`);
    wingGrad.addColorStop(1, `rgba(36, 96, 140, ${m.alpha * 0.95})`);
    ctx.fillStyle = wingGrad;
    ctx.fill();

    // Spine highlight.
    ctx.beginPath();
    ctx.moveTo(-m.w * 0.16, 0);
    ctx.quadraticCurveTo(0, -m.h * 0.08, m.w * 0.16, 0);
    ctx.strokeStyle = `rgba(190, 235, 255, ${m.alpha * 0.75})`;
    ctx.lineWidth = Math.max(1.2, m.w * 0.004);
    ctx.stroke();

    // Elegant tail.
    ctx.beginPath();
    ctx.moveTo(-m.w * 0.50, m.h * 0.02);
    ctx.bezierCurveTo(-m.w * 0.62, m.h * 0.16 + tailSway, -m.w * 0.76, m.h * 0.18 + tailSway, -m.w * 0.92, m.h * 0.03 + tailSway);
    ctx.strokeStyle = `rgba(151, 220, 255, ${m.alpha * 0.75})`;
    ctx.lineWidth = Math.max(1.5, m.w * 0.008);
    ctx.lineCap = 'round';
    ctx.stroke();

    ctx.restore();
    ctx.filter = 'none';
  }

  function animate(ts) {
    if (startTs == null) startTs = ts;
    const elapsed = ts - startTs;

    ctx.clearRect(0, 0, width, height);

    let allDone = true;

    for (const m of mantas) {
      const local = elapsed - m.delayMs;
      if (local < 0) {
        allDone = false;
        continue;
      }

      const p = Math.min(1, local / m.durationMs);
      // very smooth start/end for awe vibe
      const eased = p < 0.5
        ? 4 * p * p * p
        : 1 - Math.pow(-2 * p + 2, 3) / 2;

      const x = m.startX + (m.endX - m.startX) * eased;
      drawManta(m, x, ts);

      if (p < 1) allDone = false;
    }

    if (!allDone) {
      rafId = requestAnimationFrame(animate);
      return;
    }

    hasRun = true;

    // One-shot and done.
    canvas.style.transition = 'opacity 1000ms ease';
    canvas.style.opacity = '0';
    setTimeout(() => {
      ctx.clearRect(0, 0, width, height);
      canvas.classList.remove('active');
    }, 1050);
  }

  function runOnce() {
    if (hasRun) return;
    resize();
    buildFleet();
    canvas.classList.add('active');
    canvas.style.opacity = '1';
    startTs = null;
    if (rafId) cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(animate);
  }

  window.addEventListener('load', () => {
    // Let page settle first, then majestic pass.
    setTimeout(runOnce, 1100);
  }, { once: true });

  window.addEventListener('resize', () => {
    if (!hasRun) resize();
  }, { passive: true });
})();
