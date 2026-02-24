#!/usr/bin/env node
/**
 * Convert a GIF/video to ASCII brightness frames for the repo.box background.
 * 
 * Usage: node gif-to-ascii.js <input> <output.json> [--cols=80] [--rows=40]
 * 
 * Requires: ffmpeg
 * 
 * Output: JSON array of frames, each frame is an array of rows,
 * each row is an array of brightness values (0.0-1.0)
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
const input = args[0];
const output = args[1] || 'frames.json';
const cols = parseInt(args.find(a => a.startsWith('--cols='))?.split('=')[1] || '100');
const rows = parseInt(args.find(a => a.startsWith('--rows='))?.split('=')[1] || '50');

if (!input) {
  console.error('Usage: node gif-to-ascii.js <input.gif> [output.json] [--cols=100] [--rows=50]');
  process.exit(1);
}

const tmpDir = `/tmp/ascii-frames-${Date.now()}`;
fs.mkdirSync(tmpDir, { recursive: true });

console.log(`Extracting frames at ${cols}x${rows}...`);

// Extract frames as raw grayscale, scaled to our target size
execSync(
  `ffmpeg -i "${input}" -vf "scale=${cols}:${rows},format=gray" -f rawvideo -pix_fmt gray "${tmpDir}/frames.raw"`,
  { stdio: 'pipe' }
);

const raw = fs.readFileSync(`${tmpDir}/frames.raw`);
const frameSize = cols * rows;
const numFrames = Math.floor(raw.length / frameSize);

console.log(`Got ${numFrames} frames`);

const frames = [];
for (let f = 0; f < numFrames; f++) {
  const frame = [];
  for (let y = 0; y < rows; y++) {
    const row = [];
    for (let x = 0; x < cols; x++) {
      const brightness = raw[f * frameSize + y * cols + x] / 255;
      // Quantize to 1 decimal place to save space
      row.push(Math.round(brightness * 10) / 10);
    }
    frame.push(row);
  }
  frames.push(frame);
}

// Write output
const jsonStr = JSON.stringify(frames);
fs.writeFileSync(output, jsonStr);

// Cleanup
fs.rmSync(tmpDir, { recursive: true });

console.log(`Wrote ${numFrames} frames (${cols}x${rows}) to ${output} (${(jsonStr.length / 1024).toFixed(1)}KB)`);
