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

  let frames = null;
  let frameCanvases = null;
  let frameW = 0;
  let frameH = 0;

  const isMobile = window.innerWidth < 900;
  const fleetSize = isMobile ? 3 : 4;
  const sizeRangeVW = [0.20, 0.30];
  const baseDurationMs = isMobile ? 9800 : 10500;

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
    const centerY = height * (isMobile ? 0.46 : 0.43);

    for (let i = 0; i < fleetSize; i++) {
      const depth = i / Math.max(1, fleetSize - 1);
      const sizeVW = sizeRangeVW[0] + Math.random() * (sizeRangeVW[1] - sizeRangeVW[0]);
      const w = width * sizeVW * (1 - depth * 0.08);
      const h = w * (frameH / frameW);

      mantas.push({
        w,
        h,
        startX: -w - i * (width * 0.08),
        endX: width + w + i * (width * 0.03),
        y: centerY + (i - (fleetSize - 1) / 2) * (height * 0.08),
        phase: Math.random() * Math.PI * 2,
        alpha: 0.95 - depth * 0.25,
        glow: 0.2 - depth * 0.08,
        delayMs: i * 680,
        durationMs: baseDurationMs + i * 460,
        driftAmp: 4 + Math.random() * 4,
        frameRateScale: 0.84 + Math.random() * 0.12, // slower cadence than source
      });
    }
  }

  function preprocessFrames() {
    frameH = frames[0].length;
    frameW = frames[0][0].length;

    frameCanvases = frames.map((frame) => {
      const c = document.createElement('canvas');
      c.width = frameW;
      c.height = frameH;
      const cctx = c.getContext('2d');
      const img = cctx.createImageData(frameW, frameH);

      let p = 0;
      for (let y = 0; y < frameH; y++) {
        for (let x = 0; x < frameW; x++) {
          const v = frame[y][x];
          // Source has bright water + darker manta. Convert to dark-subject mask.
          let a = Math.max(0, (0.68 - v) / 0.68);
          a = Math.pow(a, 1.55);

          // Kill background noise aggressively.
          if (a < 0.11) a = 0;

          img.data[p++] = 115;   // R
          img.data[p++] = 196;   // G
          img.data[p++] = 244;   // B
          img.data[p++] = Math.round(a * 255);
        }
      }

      cctx.putImageData(img, 0, 0);
      return c;
    });
  }

  function drawManta(m, ts, localProgress) {
    const t = ts / 1000;
    const eased = localProgress < 0.5
      ? 4 * localProgress * localProgress * localProgress
      : 1 - Math.pow(-2 * localProgress + 2, 3) / 2;

    const x = m.startX + (m.endX - m.startX) * eased;
    const y = m.y + Math.sin(t * 0.75 + m.phase) * m.driftAmp;

    const fi = Math.floor((ts * 0.012 * m.frameRateScale) % frameCanvases.length);
    const sprite = frameCanvases[fi];

    ctx.save();
    ctx.translate(x, y);

    // Slightly bank the manta for cinematic feel.
    const bank = Math.sin(t * 0.45 + m.phase) * 0.045;
    ctx.rotate(bank);

    // Glow/atmosphere.
    ctx.globalCompositeOperation = 'lighter';
    ctx.globalAlpha = m.glow;
    ctx.drawImage(sprite, -m.w * 0.55, -m.h * 0.55, m.w * 1.1, m.h * 1.1);

    // Main body.
    ctx.globalCompositeOperation = 'source-over';
    ctx.globalAlpha = m.alpha;
    ctx.drawImage(sprite, -m.w * 0.5, -m.h * 0.5, m.w, m.h);

    ctx.restore();
    ctx.globalAlpha = 1;
    ctx.globalCompositeOperation = 'source-over';
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
      drawManta(m, ts, p);
      if (p < 1) allDone = false;
    }

    if (!allDone) {
      rafId = requestAnimationFrame(animate);
      return;
    }

    hasRun = true;

    canvas.style.transition = 'opacity 1200ms ease';
    canvas.style.opacity = '0';
    setTimeout(() => {
      ctx.clearRect(0, 0, width, height);
      canvas.classList.remove('active');
    }, 1250);
  }

  function runOnce() {
    if (hasRun || !frameCanvases) return;
    resize();
    buildFleet();
    canvas.classList.add('active');
    canvas.style.opacity = '1';
    startTs = null;
    if (rafId) cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(animate);
  }

  async function init() {
    try {
      const res = await fetch('/manta-loop.json', { cache: 'force-cache' });
      if (!res.ok) return;
      frames = await res.json();
      if (!Array.isArray(frames) || !frames.length) return;
      preprocessFrames();

      window.addEventListener('load', () => {
        setTimeout(runOnce, 1100);
      }, { once: true });

      window.addEventListener('resize', () => {
        if (!hasRun) resize();
      }, { passive: true });
    } catch (_) {
      // silently ignore if asset unavailable
    }
  }

  resize();
  init();
})();
