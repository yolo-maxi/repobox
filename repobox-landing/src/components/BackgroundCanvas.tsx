"use client";

import { useEffect, useRef } from "react";

export function BackgroundCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const isMobile =
      /Mobi|Android|iPhone|iPad/i.test(navigator.userAgent) ||
      window.innerWidth < 768;
    const fontSize = isMobile ? 10 : 7;
    const charW = fontSize * 0.6;

    let cols = 0, rows = 0;
    let bgGrid: string[][] = [];
    let bgLayer: HTMLCanvasElement | null = null;
    let wpText = "";
    let tick = 0;

    // ── Manta rays ──
    let mantaFrames: number[][][] | null = null;
    let mfW = 0, mfH = 0, totalMantaFrames = 0;
    const mantas: any[] = [];
    let mantasDone = false;
    let mantaTick = 0;

    fetch("/manta-final.json")
      .then((r) => r.json())
      .then((data) => {
        mantaFrames = data;
        totalMantaFrames = data.length;
        mfH = data[0].length;
        mfW = data[0][0].length;
        spawnFleet();
      })
      .catch(() => {});

    function spawnFleet() {
      const viewportRows = Math.ceil(window.innerHeight / fontSize);
      const fleet = isMobile
        ? [
            { scale: 1.4, startX: -mfW * 1.5, startY: viewportRows * 0.05, delay: 0, speed: 0.65 },
            { scale: 2.0, startX: -mfW * 2.2, startY: viewportRows * 0.25, delay: 3, speed: 0.4 },
            { scale: 1.0, startX: -mfW * 1.2, startY: viewportRows * 0.48, delay: 8, speed: 0.8 },
          ]
        : [
            { scale: 1.2, startX: -mfW * 1.4, startY: viewportRows * 0.02, delay: 0, speed: 0.7 },
            { scale: 2.2, startX: -mfW * 2.5, startY: viewportRows * 0.18, delay: 3, speed: 0.38 },
            { scale: 1.6, startX: -mfW * 1.8, startY: viewportRows * 0.35, delay: 6, speed: 0.55 },
            { scale: 0.9, startX: -mfW * 1.0, startY: viewportRows * 0.08, delay: 12, speed: 0.9 },
            { scale: 2.5, startX: -mfW * 3.0, startY: viewportRows * 0.42, delay: 2, speed: 0.3 },
          ];

      for (const m of fleet) {
        mantas.push({
          x: m.startX, y: m.startY, scale: m.scale, speed: m.speed,
          delay: m.delay,
          frameOffset: Math.floor(Math.random() * totalMantaFrames),
          maxAlpha: 0.15 + m.scale * 0.3,
          wobbleAmp: 0.1 + Math.random() * 0.15,
          wobbleFreq: 0.002 + Math.random() * 0.002,
          wobblePhase: Math.random() * Math.PI * 2,
          done: false,
        });
      }
    }

    function drawMantas() {
      if (!mantaFrames || mantasDone) return;
      let allDone = true;

      for (const m of mantas) {
        if (m.done) continue;
        if (mantaTick < m.delay) { allDone = false; continue; }
        allDone = false;
        m.x += m.speed;

        const wobbleY = Math.sin(mantaTick * m.wobbleFreq + m.wobblePhase) * m.wobbleAmp;
        const drawX = m.x;
        const drawY = m.y + wobbleY;
        const localTick = mantaTick - m.delay;
        const fi = Math.floor((localTick * 0.22 + m.frameOffset) % totalMantaFrames);
        const frame = mantaFrames![fi];
        const scaledW = mfW * m.scale;
        if (drawX > cols + 5) { m.done = true; continue; }

        const scaledH = mfH * m.scale;
        const gxMin = Math.max(0, Math.floor(drawX));
        const gxMax = Math.min(cols, Math.ceil(drawX + scaledW));
        const gyMin = Math.max(0, Math.floor(drawY));
        const gyMax = Math.min(rows, Math.ceil(drawY + scaledH));

        for (let gy = gyMin; gy < gyMax; gy++) {
          for (let gx = gxMin; gx < gxMax; gx++) {
            const fx = Math.floor((gx - drawX) / m.scale);
            const fy = Math.floor((gy - drawY) / m.scale);
            if (fx < 0 || fx >= mfW || fy < 0 || fy >= mfH) continue;
            const brightness = frame[fy][fx];
            if (brightness < 0.08) continue;
            const alpha = brightness * m.maxAlpha;
            if (alpha < 0.02) continue;
            const ch = bgGrid[gy]?.[gx] || ".";
            ctx!.fillStyle = `rgba(120, 220, 255, ${Math.min(alpha, 0.85)})`;
            ctx!.fillText(ch, gx * charW, gy * fontSize + fontSize);
          }
        }
      }

      if (allDone && mantas.length > 0) mantasDone = true;
      mantaTick++;
    }

    // ── Jellyfish ──
    let frames: number[][][] | null = null;
    let fW = 0, fH = 0, totalFrames = 0;
    const jellies: any[] = [];
    const MAX_JELLIES = isMobile ? 6 : 25;
    const SPAWN_INTERVAL = isMobile ? 120 : 45;
    const baseRot = (70 * Math.PI) / 180;
    let lastSpawn = 0;

    fetch("/jellyfish.json")
      .then((r) => r.json())
      .then((data) => {
        frames = data;
        totalFrames = frames!.length;
        fH = frames![0].length;
        fW = frames![0][0].length;
        const initialCount = isMobile ? 6 : 20;
        for (let i = 0; i < initialCount; i++) jellies.push(makeJelly(true));
      })
      .catch(() => {});

    function makeJelly(initial: boolean) {
      const r = Math.random();
      const scale = r < 0.9 ? 0.15 + r * 0.94 : 1.0 + Math.random() * 0.8;
      const rot = baseRot + (Math.random() - 0.5) * ((50 * Math.PI) / 180);
      let x: number, y: number;
      if (initial) {
        const foldRow = Math.ceil(window.innerHeight / fontSize);
        x = Math.random() * cols;
        y = foldRow + Math.random() * (rows - foldRow);
      } else {
        const scrollRow = Math.floor((window.scrollY || 0) / fontSize);
        const viewRows = Math.ceil(window.innerHeight / fontSize);
        x = Math.random() * cols;
        y = scrollRow + viewRows + Math.random() * 30;
      }
      const burstSpeed = (0.06 + Math.random() * 0.06) * (0.5 + scale * 0.5);
      const coastSpeed = burstSpeed * 0.15;
      const heading = -Math.PI / 2 + (Math.random() - 0.5) * 0.7;
      return {
        x, y, scale, heading,
        renderRot: rot, cosR: Math.cos(rot), sinR: Math.sin(rot),
        burstSpeed, coastSpeed, speed: coastSpeed,
        fleeing: 0,
        frameOffset: Math.floor(Math.random() * totalFrames),
        maxAlpha: 0.1 + scale * 0.5,
        phase: Math.random() * Math.PI * 2,
        swayAmp: 0.2 + Math.random() * 0.3,
        swayFreq: 0.003 + Math.random() * 0.004,
        contracting: false,
      };
    }

    // ── Cursor tracking ──
    let mouseGx = -1000, mouseGy = -1000;
    const CURSOR_GLOW_RADIUS = 12;
    const JELLY_FLEE_RADIUS = 18;

    const onMouseMove = (e: MouseEvent) => {
      mouseGx = e.clientX / charW;
      mouseGy = (e.clientY + (window.scrollY || 0)) / fontSize;
    };
    const onMouseLeave = () => { mouseGx = -1000; mouseGy = -1000; };
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseleave", onMouseLeave);

    // ── Grid ──
    fetch("/whitepaper.txt")
      .then((r) => r.text())
      .then((t) => { wpText = t.replace(/\s+/g, " ").trim(); if (bgGrid.length) rebuildBgLayer(); })
      .catch(() => {});

    function initGrid() {
      const docH = Math.max(document.body.scrollHeight, document.documentElement.scrollHeight, window.innerHeight);
      canvas!.width = window.innerWidth;
      canvas!.height = docH;
      cols = Math.ceil(canvas!.width / charW);
      rows = Math.ceil(docH / fontSize);
      bgGrid = [];
      let idx = 0;
      for (let y = 0; y < rows; y++) {
        bgGrid[y] = [];
        for (let x = 0; x < cols; x++) {
          bgGrid[y][x] = wpText.length > 0
            ? wpText[idx++ % wpText.length]
            : String.fromCharCode(33 + Math.floor(Math.random() * 94));
        }
      }
      rebuildBgLayer();
    }

    function rebuildBgLayer() {
      if (!cols || !rows) return;
      if (wpText.length > 0) {
        let idx = 0;
        for (let y = 0; y < rows; y++) {
          if (!bgGrid[y]) bgGrid[y] = [];
          for (let x = 0; x < cols; x++) { bgGrid[y][x] = wpText[idx++ % wpText.length]; }
        }
      }
      bgLayer = document.createElement("canvas");
      bgLayer.width = canvas!.width;
      bgLayer.height = canvas!.height;
      const bgCtx = bgLayer.getContext("2d")!;
      bgCtx.font = `${fontSize}px JetBrains Mono, monospace`;
      bgCtx.fillStyle = "rgba(79, 195, 247, 0.025)";
      for (let y = 0; y < rows; y++) {
        for (let x = 0; x < cols; x++) {
          bgCtx.fillText(bgGrid[y][x], x * charW, y * fontSize + fontSize);
        }
      }
    }

    // ── Main draw loop ──
    function draw() {
      ctx!.clearRect(0, 0, canvas!.width, canvas!.height);
      if (bgLayer) ctx!.drawImage(bgLayer, 0, 0);
      ctx!.font = `${fontSize}px JetBrains Mono, monospace`;

      // Cursor glow
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
              ctx!.fillStyle = `rgba(79, 195, 247, ${0.03 + intensity * 0.18})`;
              ctx!.fillText(bgGrid[gy]?.[gx] || ".", gx * charW, gy * fontSize + fontSize);
            }
          }
        }
      }

      // Mantas
      drawMantas();

      // Jellyfish
      if (frames) {
        if (tick - lastSpawn > SPAWN_INTERVAL && jellies.length < MAX_JELLIES) {
          jellies.push(makeJelly(false));
          lastSpawn = tick;
        }

        for (let ji = jellies.length - 1; ji >= 0; ji--) {
          const j = jellies[ji];
          const fi = Math.floor((tick * 0.12 + j.frameOffset) % totalFrames);
          const frame = frames![fi];
          const prevFi = (fi - 1 + totalFrames) % totalFrames;

          // Contraction detection
          let curSum = 0, prevSum = 0;
          const sRows = [Math.floor(fH * 0.2), Math.floor(fH * 0.3), Math.floor(fH * 0.4)];
          for (const sy of sRows) {
            for (let sx = 0; sx < fW; sx += 3) {
              curSum += frame[sy][sx];
              prevSum += frames![prevFi][sy][sx];
            }
          }
          j.contracting = curSum < prevSum * 0.98;

          // Cursor flee
          const dxC = j.x - mouseGx, dyC = j.y - mouseGy;
          const distC = Math.sqrt(dxC * dxC + dyC * dyC);
          const MAX_TURN = 0.05;
          const BASE_HEADING = -Math.PI / 2;
          const HEADING_LIMIT = Math.PI / 6;

          if (distC < JELLY_FLEE_RADIUS && distC > 0.1) {
            let desiredHeading = Math.atan2(dyC, dxC);
            let diffFromBase = desiredHeading - BASE_HEADING;
            while (diffFromBase > Math.PI) diffFromBase -= 2 * Math.PI;
            while (diffFromBase < -Math.PI) diffFromBase += 2 * Math.PI;
            if (diffFromBase > HEADING_LIMIT) desiredHeading = BASE_HEADING + HEADING_LIMIT;
            if (diffFromBase < -HEADING_LIMIT) desiredHeading = BASE_HEADING - HEADING_LIMIT;
            let diff = desiredHeading - j.heading;
            while (diff > Math.PI) diff -= 2 * Math.PI;
            while (diff < -Math.PI) diff += 2 * Math.PI;
            j.heading += Math.max(-MAX_TURN, Math.min(MAX_TURN, diff));
            j.fleeing = 30;
            j.frameOffset += 0.5;
          }

          // Speed
          let speed: number;
          if (j.fleeing > 0) { speed = j.burstSpeed * 2.5; j.fleeing--; }
          else { speed = j.contracting ? j.burstSpeed : j.coastSpeed; }

          j.x += Math.cos(j.heading) * speed;
          j.y += Math.sin(j.heading) * speed;

          // Steer back to default
          if (j.fleeing <= 0) {
            const defH = -Math.PI / 2 + Math.sin(tick * 0.001 + j.phase) * 0.15;
            let d = defH - j.heading;
            while (d > Math.PI) d -= 2 * Math.PI;
            while (d < -Math.PI) d += 2 * Math.PI;
            j.heading += d * 0.02;
          }
          // Clamp heading
          {
            let d = j.heading - -Math.PI / 2;
            while (d > Math.PI) d -= 2 * Math.PI;
            while (d < -Math.PI) d += 2 * Math.PI;
            if (d > Math.PI / 6) j.heading = -Math.PI / 2 + Math.PI / 6;
            if (d < -Math.PI / 6) j.heading = -Math.PI / 2 - Math.PI / 6;
          }

          j.renderRot = j.heading + Math.PI / 2 + baseRot - 0.3;
          j.cosR = Math.cos(j.renderRot);
          j.sinR = Math.sin(j.renderRot);

          const sway = Math.sin(tick * j.swayFreq + j.phase) * j.swayAmp;
          const perpX = -Math.sin(j.heading), perpY = Math.cos(j.heading);
          const drawX = j.x + sway * perpX, drawY = j.y + sway * perpY;

          // Remove when off viewport
          const viewTop = Math.floor((window.scrollY || 0) / fontSize) - 40;
          if (drawY < viewTop || drawX > cols + 20 || drawX < -20 || drawY > rows + 20) {
            jellies.splice(ji, 1);
            continue;
          }

          // Cursor glow boost
          let glowBoost = 1;
          if (!isMobile) {
            const jd = Math.sqrt((drawX - mouseGx) ** 2 + (drawY - mouseGy) ** 2);
            if (jd < CURSOR_GLOW_RADIUS * 2) glowBoost = 1 + (1 - jd / (CURSOR_GLOW_RADIUS * 2)) * 1.5;
          }

          const halfW = (fW * j.scale) / 2, halfH = (fH * j.scale) / 2;
          const gxMin = Math.max(0, Math.floor(drawX - halfW - 2));
          const gxMax = Math.min(cols, Math.ceil(drawX + halfW + 2));
          const gyMin = Math.max(0, Math.floor(drawY - halfH - 2));
          const gyMax = Math.min(rows, Math.ceil(drawY + halfH + 2));

          for (let gy = gyMin; gy < gyMax; gy++) {
            for (let gx = gxMin; gx < gxMax; gx++) {
              const relX = gx - drawX, relY = gy - drawY;
              const localX = relX * j.cosR + relY * j.sinR;
              const localY = -relX * j.sinR + relY * j.cosR;
              const fx = Math.floor(localX / j.scale + fW / 2);
              const fy = Math.floor(localY / j.scale + fH / 2);
              if (fx < 0 || fx >= fW || fy < 0 || fy >= fH) continue;
              const brightness = frame[fy][fx];
              if (brightness < 0.08) continue;
              const alpha = brightness * j.maxAlpha * glowBoost;
              if (alpha < 0.02) continue;
              ctx!.fillStyle = `rgba(79, 195, 247, ${Math.min(alpha, 0.9)})`;
              ctx!.fillText(bgGrid[gy]?.[gx] || ".", gx * charW, gy * fontSize + fontSize);
            }
          }
        }
      }

      tick++;
      animRef.current = requestAnimationFrame(draw);
    }

    initGrid();
    draw();

    const onResize = () => initGrid();
    window.addEventListener("resize", onResize);
    window.addEventListener("load", () => setTimeout(initGrid, 100));

    return () => {
      cancelAnimationFrame(animRef.current);
      window.removeEventListener("resize", onResize);
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseleave", onMouseLeave);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        zIndex: 1,
        pointerEvents: "none",
        opacity: 0.8,
      }}
    />
  );
}
