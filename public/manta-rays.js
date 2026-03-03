(() => {
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (prefersReducedMotion) return;

  let canvas = document.getElementById('manta-pass-canvas');
  if (!canvas) {
    canvas = document.createElement('canvas');
    canvas.id = 'manta-pass-canvas';
    canvas.setAttribute('aria-hidden', 'true');
    canvas.style.position = 'absolute';
    canvas.style.top = '0';
    canvas.style.left = '0';
    canvas.style.width = '100%';
    canvas.style.pointerEvents = 'none';
    canvas.style.zIndex = '1';
    canvas.style.opacity = '0';
    canvas.style.transition = 'opacity 1200ms ease';
    document.body.appendChild(canvas);
  }

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
  const sizeRangeVW = isMobile ? [0.22, 0.29] : [0.20, 0.30];
  const baseDurationMs = 11000;
  const mantas = [];

  function documentHeight() {
    return Math.max(
      document.body.scrollHeight,
      document.documentElement.scrollHeight,
      document.body.offsetHeight,
      document.documentElement.offsetHeight,
      window.innerHeight
    );
  }

  function resize() {
    width = Math.max(1, window.innerWidth);
    height = Math.max(1, documentHeight());
    dpr = Math.min(window.devicePixelRatio || 1, 2);

    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
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
          // dark manta in source -> bright cyan in output
          let a = Math.max(0, (0.58 - v) / 0.58);
          a = Math.pow(a, 1.9);
          if (a < 0.08) a = 0;

          img.data[p++] = 120;
          img.data[p++] = 214;
          img.data[p++] = 255;
          img.data[p++] = Math.round(a * 255);
        }
      }

      cctx.putImageData(img, 0, 0);
      return c;
    });
  }

  function buildFleet() {
    mantas.length = 0;
    const yAnchors = isMobile ? [0.18, 0.36, 0.62] : [0.14, 0.30, 0.52, 0.76];

    for (let i = 0; i < fleetSize; i++) {
      const depth = i / Math.max(1, fleetSize - 1);
      const sizeVW = sizeRangeVW[0] + Math.random() * (sizeRangeVW[1] - sizeRangeVW[0]);
      const w = width * sizeVW * (1 - depth * 0.08);
      const h = w * (frameH / frameW);

      mantas.push({
        w,
        h,
        startX: -w - i * (width * 0.10),
        endX: width + w + i * (width * 0.04),
        y: Math.min(height - h * 0.55, Math.max(h * 0.55, height * yAnchors[i])),
        phase: Math.random() * Math.PI * 2,
        alpha: 0.92 - depth * 0.22,
        glow: 0.22 - depth * 0.08,
        delayMs: i * 680,
        durationMs: baseDurationMs + i * 520,
        driftAmp: 6 + Math.random() * 6,
        frameRateScale: 0.84 + Math.random() * 0.11,
      });
    }
  }

  function drawManta(m, ts, progress) {
    const t = ts / 1000;
    const eased = progress < 0.5
      ? 4 * progress * progress * progress
      : 1 - Math.pow(-2 * progress + 2, 3) / 2;

    const x = m.startX + (m.endX - m.startX) * eased;
    const y = m.y + Math.sin(t * 0.65 + m.phase) * m.driftAmp;

    const fi = Math.floor((ts * 0.012 * m.frameRateScale) % frameCanvases.length);
    const sprite = frameCanvases[fi];

    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(Math.sin(t * 0.4 + m.phase) * 0.04);

    ctx.globalCompositeOperation = 'lighter';
    ctx.globalAlpha = m.glow;
    ctx.drawImage(sprite, -m.w * 0.58, -m.h * 0.58, m.w * 1.16, m.h * 1.16);

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
    canvas.style.opacity = '0';
    setTimeout(() => {
      ctx.clearRect(0, 0, width, height);
    }, 1300);
  }

  function runOnce() {
    if (hasRun || !frameCanvases) return;
    resize();
    buildFleet();
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

      window.addEventListener('load', () => setTimeout(runOnce, 1200), { once: true });
      window.addEventListener('resize', () => { if (!hasRun) resize(); }, { passive: true });
    } catch (_) {
      // no-op
    }
  }

  resize();
  init();
})();
