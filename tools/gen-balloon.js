#!/usr/bin/env node
/**
 * Generate a high-detail hot air balloon as brightness frames.
 * Uses math to draw a clean balloon shape with internal patterns.
 * Outputs JSON brightness grid for the background animation.
 */

const fs = require('fs');
const cols = 80;
const rows = 100;
const numFrames = 30; // for gentle animation

function generateFrame(frameIdx) {
  const grid = [];
  const t = (frameIdx / numFrames) * Math.PI * 2;
  
  // Balloon parameters
  const cx = cols / 2;
  const envTop = 8;     // top of envelope
  const envBot = 55;    // bottom of envelope
  const envMidY = (envTop + envBot) / 2;
  const envHeight = envBot - envTop;
  
  // Basket
  const basketTop = 70;
  const basketBot = 80;
  const basketW = 10;
  
  for (let y = 0; y < rows; y++) {
    const row = [];
    for (let x = 0; x < cols; x++) {
      let brightness = 0;
      
      // === ENVELOPE (balloon shape) ===
      if (y >= envTop && y <= envBot) {
        // Envelope radius at this y (elliptical shape, wider in upper-middle)
        const normY = (y - envTop) / envHeight; // 0 at top, 1 at bottom
        
        // Shape: wide at 0.3-0.5, narrowing at top and bottom
        let radius;
        if (normY < 0.1) {
          // Crown - small circle at top
          radius = normY * 280;
        } else if (normY < 0.6) {
          // Upper body - widest part
          const t2 = (normY - 0.1) / 0.5;
          radius = 28 + Math.sin(t2 * Math.PI) * 10;
        } else {
          // Lower body - narrows to opening
          const t2 = (normY - 0.6) / 0.4;
          radius = 28 * (1 - t2 * 0.7);
        }
        
        const dx = Math.abs(x - cx);
        
        if (dx <= radius) {
          // Inside envelope
          const edgeDist = radius - dx;
          
          // Base brightness for envelope body
          brightness = 0.5;
          
          // Edge highlight (brighter at edges)
          if (edgeDist < 2) {
            brightness = 0.8;
          }
          
          // Vertical gore lines (the panels on a balloon)
          const numGores = 8;
          const angle = Math.atan2(x - cx, y - envMidY);
          const gorePhase = (angle * numGores / Math.PI) % 1;
          if (Math.abs(gorePhase) < 0.08 || Math.abs(gorePhase - 1) < 0.08) {
            brightness = 0.7; // gore seam lines
          }
          
          // Horizontal bands
          if (normY > 0.15 && normY < 0.55) {
            const bandIdx = Math.floor(normY * 8);
            if (bandIdx % 2 === 0) {
              brightness = Math.max(brightness, 0.55);
            } else {
              brightness = Math.min(brightness, 0.45);
            }
          }
          
          // Gentle animation: slight brightness wave
          const wave = Math.sin(t + normY * 4 + (x - cx) * 0.2) * 0.08;
          brightness += wave;
          
          // Crown highlight
          if (normY < 0.08) {
            brightness = 0.9;
          }
        }
      }
      
      // === ROPES (connecting envelope to basket) ===
      if (y > envBot && y < basketTop) {
        const ropeProgress = (y - envBot) / (basketTop - envBot);
        // 4 ropes from envelope bottom to basket corners
        const envBottomRadius = 28 * 0.3; // radius at envelope bottom
        const ropePositions = [
          cx - envBottomRadius * 0.8 + ropeProgress * (cx - basketW/2 - (cx - envBottomRadius * 0.8)),
          cx - envBottomRadius * 0.3 + ropeProgress * (cx - basketW/4 - (cx - envBottomRadius * 0.3)),
          cx + envBottomRadius * 0.3 + ropeProgress * (cx + basketW/4 - (cx + envBottomRadius * 0.3)),
          cx + envBottomRadius * 0.8 + ropeProgress * (cx + basketW/2 - (cx + envBottomRadius * 0.8)),
        ];
        
        for (const ropeX of ropePositions) {
          if (Math.abs(x - ropeX) < 0.8) {
            brightness = 0.4;
          }
        }
        
        // Burner flame (animated)
        if (y > envBot + 2 && y < envBot + 8) {
          const flameDx = Math.abs(x - cx);
          const flameHeight = 6;
          const flameY = y - envBot - 2;
          const flameWidth = 3 * (1 - flameY / flameHeight);
          const flameFlicker = Math.sin(t * 3 + flameY * 2) * 0.5;
          if (flameDx < flameWidth + flameFlicker) {
            brightness = 0.9 - flameY * 0.1;
          }
        }
      }
      
      // === BASKET ===
      if (y >= basketTop && y <= basketBot) {
        const dx = Math.abs(x - cx);
        if (dx <= basketW / 2) {
          brightness = 0.6;
          // Weave pattern
          const weaveX = Math.floor(x) % 3;
          const weaveY = Math.floor(y) % 2;
          if ((weaveX + weaveY) % 2 === 0) {
            brightness = 0.5;
          }
          // Basket edge
          if (dx >= basketW / 2 - 1 || y === basketTop || y === basketBot) {
            brightness = 0.7;
          }
        }
      }
      
      // === SANDBAGS (hanging from basket) ===
      if (y > basketBot && y < basketBot + 5) {
        const bagPositions = [cx - 3, cx + 3];
        for (const bx of bagPositions) {
          const dx = Math.abs(x - bx);
          const dy = y - basketBot;
          if (dy === 1 && dx < 0.5) brightness = 0.3; // rope
          if (dy >= 2 && dy <= 4 && dx < 1.5) brightness = 0.5; // bag
        }
      }
      
      row.push(Math.max(0, Math.min(1, Math.round(brightness * 10) / 10)));
    }
    grid.push(row);
  }
  return grid;
}

const frames = [];
for (let i = 0; i < numFrames; i++) {
  frames.push(generateFrame(i));
}

const output = process.argv[2] || '/home/xiko/repobox/public/balloon-frames.json';
const json = JSON.stringify(frames);
fs.writeFileSync(output, json);
console.log(`Generated ${numFrames} frames (${cols}x${rows}) → ${output} (${(json.length/1024).toFixed(1)}KB)`);

// Preview first frame
const chars = ' .:-=+*#%@';
const f = frames[0];
for (const row of f) {
  console.log(row.map(v => chars[Math.floor(v * (chars.length-1))]).join(''));
}
