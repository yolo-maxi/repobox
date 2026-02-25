// Ambient ASCII background: char grid + jellyfish
(function() {
  const canvas = document.getElementById('bg-canvas');
  const ctx = canvas.getContext('2d');
  
  // Mobile detection & performance scaling
  const isMobile = /Mobi|Android|iPhone|iPad/i.test(navigator.userAgent) || window.innerWidth < 768;
  const fontSize = isMobile ? 10 : 7;  // bigger chars = fewer cells = less work
  const charW = fontSize * 0.6;
  let lastFrameTime = 0;
  
  let cols, rows, bgGrid, bgLayer;
  let wpText = '';
  
  // Load Bitcoin whitepaper for the background grid
  fetch('/whitepaper.txt')
    .then(r => r.text())
    .then(t => { wpText = t.replace(/\s+/g, ' ').trim(); if (bgGrid) rebuildBgLayer(); });
  
  function initGrid() {
    const docH = Math.max(document.body.scrollHeight, document.documentElement.scrollHeight, window.innerHeight);
    canvas.width = window.innerWidth;
    canvas.height = docH;
    cols = Math.ceil(canvas.width / charW);
    rows = Math.ceil(docH / fontSize);
    bgGrid = [];
    let idx = 0;
    for (let y = 0; y < rows; y++) {
      bgGrid[y] = [];
      for (let x = 0; x < cols; x++) {
        if (wpText.length > 0) {
          bgGrid[y][x] = wpText[idx % wpText.length];
          idx++;
        } else {
          bgGrid[y][x] = String.fromCharCode(33 + Math.floor(Math.random() * 94));
        }
      }
    }
    rebuildBgLayer();
  }
  
  function rebuildBgLayer() {
    if (!cols || !rows) return;
    // Rebuild grid text if whitepaper loaded after initial grid
    if (wpText.length > 0) {
      let idx = 0;
      for (let y = 0; y < rows; y++) {
        if (!bgGrid[y]) bgGrid[y] = [];
        for (let x = 0; x < cols; x++) {
          bgGrid[y][x] = wpText[idx % wpText.length];
          idx++;
        }
      }
    }
    // Pre-render background to offscreen canvas
    bgLayer = document.createElement('canvas');
    bgLayer.width = canvas.width;
    bgLayer.height = canvas.height;
    const bgCtx = bgLayer.getContext('2d');
    bgCtx.font = `${fontSize}px JetBrains Mono, monospace`;
    bgCtx.fillStyle = 'rgba(79, 195, 247, 0.025)';
    for (let y = 0; y < rows; y++) {
      for (let x = 0; x < cols; x++) {
        bgCtx.fillText(bgGrid[y][x], x * charW, y * fontSize + fontSize);
      }
    }
  }
  
  // Animated jellyfish — multiple instances from video brightness frames
  // Rotated +70° (jellyfish tilted), moving in bursts toward ~1 o'clock (up-right ~30° from vertical)
  let frames = null;
  let fW = 0, fH = 0, totalFrames = 0;
  const jellies = [];
  const MAX_JELLIES = isMobile ? 6 : 25;
  const SPAWN_INTERVAL = isMobile ? 120 : 45;
  
  // Base movement direction: ~1 o'clock = 30° from vertical
  const moveAngle = Math.PI / 6;
  const moveDirX = Math.sin(moveAngle);
  const moveDirY = -Math.cos(moveAngle);
  
  // Base rotation: +70° clockwise (per-jelly variation added in makeJelly)
  const baseRot = 70 * Math.PI / 180;
  
  // Pre-rendered jellyfish frames as offscreen canvases
  let frameCanvases = []; // one offscreen canvas per animation frame
  
  fetch('/jellyfish.json')
    .then(r => r.json())
    .then(data => {
      frames = data;
      totalFrames = frames.length;
      fH = frames[0].length;
      fW = frames[0][0].length;
      
      // No pre-rendered canvases needed — jellyfish modulate the background grid directly
      
      // Initial batch
      const initialCount = isMobile ? 3 : 12;
      for (let i = 0; i < initialCount; i++) jellies.push(makeJelly(true));
    });
  
  function makeJelly(initial) {
    // Scale: mostly 0.15–1.0, but occasionally (10% chance) a big one up to 1.8×
    const r = Math.random();
    const scale = r < 0.9 ? (0.15 + r * 0.94) : (1.0 + Math.random() * 0.8);
    // Per-jellyfish rotation: base ±25°
    const rot = baseRot + (Math.random() - 0.5) * (50 * Math.PI / 180);
    const cosR = Math.cos(rot);
    const sinR = Math.sin(rot);
    // Rotated bounding box
    const diagW = Math.ceil(Math.abs(fW * cosR) + Math.abs(fH * sinR)) * scale;
    const diagH = Math.ceil(Math.abs(fW * sinR) + Math.abs(fH * cosR)) * scale;
    
    // Spawn from bottom-left area, drift toward upper-right
    let x, y;
    if (initial) {
      x = Math.random() * cols;
      y = Math.random() * rows;
    } else {
      // Spawn from bottom edge (they only go up)
      x = Math.random() * cols;
      y = rows + Math.random() * 30;
    }
    
    // Burst movement: jellyfish contract to move, then coast
    // We'll track animation frame and derive burst from contraction phase
    const burstSpeed = (0.06 + Math.random() * 0.06) * (0.5 + scale * 0.5);
    const coastSpeed = burstSpeed * 0.15; // slow drift between pulses
    
    // Heading: roughly straight up (±20° variation within the ±30° limit)
    const heading = -Math.PI / 2 + (Math.random() - 0.5) * 0.7;
    
    return {
      x, y, scale,
      heading,       // current movement direction
      renderRot: rot, // visual rotation (smoothly follows heading + offset)
      cosR, sinR,    // cached for rendering (updated when renderRot changes)
      burstSpeed, coastSpeed,
      speed: coastSpeed, // current speed (ramps up on flee)
      fleeing: 0,    // frames of flee remaining
      frameOffset: Math.floor(Math.random() * totalFrames),
      maxAlpha: 0.1 + scale * 0.5,
      phase: Math.random() * Math.PI * 2,
      swayAmp: 0.2 + Math.random() * 0.3,
      swayFreq: 0.003 + Math.random() * 0.004,
      prevBrightSum: 0,
      contracting: false,
    };
  }
  
  let tick = 0;
  let lastSpawn = 0;
  
  // === Deep ocean elements ===
  
  // Removed: plankton, bubbles, light rays — didn't look good
  
  // Cursor tracking (in grid coordinates)
  let mouseGx = -1000, mouseGy = -1000;
  const CURSOR_GLOW_RADIUS = 12; // grid cells
  const JELLY_FLEE_RADIUS = 18;  // grid cells — detection range
  const JELLY_FLEE_FORCE = 0.8;  // impulse strength
  
  document.addEventListener('mousemove', (e) => {
    const scrollY = window.scrollY || window.pageYOffset;
    mouseGx = e.clientX / charW;
    mouseGy = (e.clientY + scrollY) / fontSize;
  });
  document.addEventListener('mouseleave', () => {
    mouseGx = -1000; mouseGy = -1000;
  });
  
  function draw() {
    
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (bgLayer) ctx.drawImage(bgLayer, 0, 0);
    
    ctx.font = `${fontSize}px JetBrains Mono, monospace`;
    
    // (ocean elements removed — jellyfish are enough)
    
    // Cursor glow: brighten grid chars near mouse (desktop only)
    if (!isMobile && mouseGx > -500) {
      const glowR2 = CURSOR_GLOW_RADIUS * CURSOR_GLOW_RADIUS;
      const gxMin = Math.max(0, Math.floor(mouseGx - CURSOR_GLOW_RADIUS));
      const gxMax = Math.min(cols, Math.ceil(mouseGx + CURSOR_GLOW_RADIUS));
      const gyMin = Math.max(0, Math.floor(mouseGy - CURSOR_GLOW_RADIUS));
      const gyMax = Math.min(rows, Math.ceil(mouseGy + CURSOR_GLOW_RADIUS));
      for (let gy = gyMin; gy < gyMax; gy++) {
        for (let gx = gxMin; gx < gxMax; gx++) {
          const dx = gx - mouseGx, dy = gy - mouseGy;
          const d2 = dx * dx + dy * dy;
          if (d2 < glowR2) {
            const intensity = 1 - Math.sqrt(d2) / CURSOR_GLOW_RADIUS;
            const alpha = 0.03 + intensity * 0.18;
            ctx.fillStyle = `rgba(79, 195, 247, ${alpha})`;
            ctx.fillText(bgGrid[gy]?.[gx] || '.', gx * charW, gy * fontSize + fontSize);
          }
        }
      }
    }
    
    if (!frames) { requestAnimationFrame(draw); tick++; return; }
    
    // Spawn new jellyfish periodically
    if (tick - lastSpawn > SPAWN_INTERVAL && jellies.length < MAX_JELLIES) {
      jellies.push(makeJelly(false));
      lastSpawn = tick;
    }
    
    for (let ji = jellies.length - 1; ji >= 0; ji--) {
      const j = jellies[ji];
      
      // Current animation frame
      const fi = Math.floor((tick * 0.12 + j.frameOffset) % totalFrames);
      const frame = frames[fi];
      
      // Detect contraction: compare brightness of current vs previous frame
      const prevFi = (fi - 1 + totalFrames) % totalFrames;
      let curSum = 0, prevSum = 0;
      const sampleRows = [Math.floor(fH * 0.2), Math.floor(fH * 0.3), Math.floor(fH * 0.4)];
      for (const sy of sampleRows) {
        for (let sx = 0; sx < fW; sx += 3) {
          curSum += frame[sy][sx];
          prevSum += frames[prevFi][sy][sx];
        }
      }
      j.contracting = curSum < prevSum * 0.98;
      
      // Cursor flee: rotate toward "away from cursor" direction, then thrust forward
      const dxCursor = j.x - mouseGx;
      const dyCursor = j.y - mouseGy;
      const distCursor = Math.sqrt(dxCursor * dxCursor + dyCursor * dyCursor);
      const MAX_TURN = 0.05; // ~3° per frame max turn rate (±30° over ~10 frames)
      
      // Absolute heading limits: jellyfish can only face within ±30° of straight up (-PI/2)
      const BASE_HEADING = -Math.PI / 2; // straight up
      const HEADING_LIMIT = Math.PI / 6; // ±30°
      
      if (distCursor < JELLY_FLEE_RADIUS && distCursor > 0.1) {
        // Desired heading: away from cursor
        const desiredHeading = Math.atan2(dyCursor, dxCursor);
        // Clamp desired heading to allowed range
        let clampedDesired = desiredHeading;
        let diffFromBase = clampedDesired - BASE_HEADING;
        while (diffFromBase > Math.PI) diffFromBase -= 2 * Math.PI;
        while (diffFromBase < -Math.PI) diffFromBase += 2 * Math.PI;
        if (diffFromBase > HEADING_LIMIT) clampedDesired = BASE_HEADING + HEADING_LIMIT;
        if (diffFromBase < -HEADING_LIMIT) clampedDesired = BASE_HEADING - HEADING_LIMIT;
        
        // Turn toward clamped desired heading
        let diff = clampedDesired - j.heading;
        while (diff > Math.PI) diff -= 2 * Math.PI;
        while (diff < -Math.PI) diff += 2 * Math.PI;
        const turn = Math.max(-MAX_TURN, Math.min(MAX_TURN, diff));
        j.heading += turn;
        j.fleeing = 30;
        j.frameOffset += 0.5;
      }
      
      // Determine speed: fleeing > contracting burst > coast
      let speed;
      if (j.fleeing > 0) {
        speed = j.burstSpeed * 2.5; // panicked speed
        j.fleeing--;
      } else {
        speed = j.contracting ? j.burstSpeed : j.coastSpeed;
      }
      
      // Move forward along heading
      j.x += Math.cos(j.heading) * speed;
      j.y += Math.sin(j.heading) * speed;
      
      // Gradually steer heading back toward default direction when not fleeing
      if (j.fleeing <= 0) {
        const defaultHeading = -Math.PI / 2 + (Math.sin(tick * 0.001 + j.phase) * 0.15); // ~straight up with tiny drift
        let diff = defaultHeading - j.heading;
        while (diff > Math.PI) diff -= 2 * Math.PI;
        while (diff < -Math.PI) diff += 2 * Math.PI;
        j.heading += diff * 0.02;
      }
      // Hard clamp: never exceed ±30° from straight up
      {
        let diffFromUp = j.heading - (-Math.PI / 2);
        while (diffFromUp > Math.PI) diffFromUp -= 2 * Math.PI;
        while (diffFromUp < -Math.PI) diffFromUp += 2 * Math.PI;
        const LIMIT = Math.PI / 6;
        if (diffFromUp > LIMIT) j.heading = -Math.PI / 2 + LIMIT;
        if (diffFromUp < -LIMIT) j.heading = -Math.PI / 2 - LIMIT;
      }
      
      // Update visual rotation to follow heading (with offset to match jellyfish image orientation)
      // The jellyfish image points "up" in source, so renderRot maps heading to visual angle
      j.renderRot = j.heading + Math.PI / 2 + baseRot - 0.3; // tune offset to look right
      j.cosR = Math.cos(j.renderRot);
      j.sinR = Math.sin(j.renderRot);
      
      // Gentle perpendicular sway
      const sway = Math.sin(tick * j.swayFreq + j.phase) * j.swayAmp;
      const perpX = -Math.sin(j.heading); // perpendicular to heading
      const perpY = Math.cos(j.heading);
      const drawX = j.x + sway * perpX;
      const drawY = j.y + sway * perpY;
      
      // Recalc bounding box from current rotation
      const diagW = Math.ceil(Math.abs(fW * j.cosR) + Math.abs(fH * j.sinR)) * j.scale;
      const diagH = Math.ceil(Math.abs(fW * j.sinR) + Math.abs(fH * j.cosR)) * j.scale;
      
      // Remove when fully off screen
      if (drawY + diagH < -20 || drawX > cols + 20 || drawX + diagW < -20 || drawY > rows + 20) {
        jellies.splice(ji, 1);
        continue;
      }
      
      // Render jellyfish by modulating background grid brightness
      // For each grid cell covered by the jellyfish, sample brightness and redraw the char brighter
      
      // Cursor glow boost
      let glowBoost = 1;
      if (!isMobile) {
        const jellyDist = Math.sqrt((drawX - mouseGx) ** 2 + (drawY - mouseGy) ** 2);
        if (jellyDist < CURSOR_GLOW_RADIUS * 2) {
          glowBoost = 1 + (1 - jellyDist / (CURSOR_GLOW_RADIUS * 2)) * 1.5;
        }
      }
      
      // Bounding box in grid coords
      const scaledW = fW * j.scale;
      const scaledH = fH * j.scale;
      const halfW = scaledW / 2;
      const halfH = scaledH / 2;
      
      // Scan grid cells in the bounding box area
      const gxMin = Math.max(0, Math.floor(drawX - halfW - 2));
      const gxMax = Math.min(cols, Math.ceil(drawX + halfW + 2));
      const gyMin = Math.max(0, Math.floor(drawY - halfH - 2));
      const gyMax = Math.min(rows, Math.ceil(drawY + halfH + 2));
      
      for (let gy = gyMin; gy < gyMax; gy++) {
        for (let gx = gxMin; gx < gxMax; gx++) {
          // Transform grid position to jellyfish frame coordinates (with rotation)
          const relX = gx - drawX;
          const relY = gy - drawY;
          // Inverse rotation to get frame-local coords
          const localX = relX * j.cosR + relY * j.sinR;
          const localY = -relX * j.sinR + relY * j.cosR;
          // Scale to frame pixel coords
          const fx = Math.floor(localX / j.scale + fW / 2);
          const fy = Math.floor(localY / j.scale + fH / 2);
          
          if (fx < 0 || fx >= fW || fy < 0 || fy >= fH) continue;
          const brightness = frame[fy][fx];
          if (brightness < 0.08) continue;
          
          const alpha = brightness * j.maxAlpha * glowBoost;
          if (alpha < 0.02) continue;
          
          const ch = bgGrid[gy]?.[gx] || '.';
          ctx.fillStyle = `rgba(79, 195, 247, ${Math.min(alpha, 0.9)})`;
          ctx.fillText(ch, gx * charW, gy * fontSize + fontSize);
        }
      }
    }
    
    tick++;
    requestAnimationFrame(draw);
  }
  
  initGrid();
  draw();
  window.addEventListener('resize', () => { initGrid(); });
  window.addEventListener('load', () => { setTimeout(initGrid, 100); });
})();
