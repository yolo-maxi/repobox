// Rasterise the auth.repo.box PWA icons from the checked-in SVG sources with
// Playwright's Chromium (transparent background, exact pixel sizes).
//   NODE_PATH=…/node_modules node scripts/pwa-icons.js [chrome-executable]
const fs = require('fs');
const path = require('path');
const { chromium } = require('playwright');
const dir = path.join(__dirname, '..', 'assets', 'pwa');
const jobs = [
  ['icon.svg', 'icon-192.png', 192],
  ['icon.svg', 'icon-512.png', 512],
  ['icon-maskable.svg', 'maskable-192.png', 192],
  ['icon-maskable.svg', 'maskable-512.png', 512],
  // iOS masks the corners itself, so it gets the full-bleed artwork.
  ['icon-maskable.svg', 'apple-touch-icon.png', 180],
];
(async () => {
  const b = await chromium.launch(process.argv[2] ? { executablePath: process.argv[2] } : {});
  const page = await b.newPage();
  for (const [src, out, px] of jobs) {
    const svg = fs.readFileSync(path.join(dir, src), 'utf8');
    await page.setViewportSize({ width: px, height: px });
    await page.setContent(`<html><body style="margin:0;background:transparent">${svg.replace('<svg ', `<svg width="${px}" height="${px}" `)}</body></html>`);
    await page.screenshot({ path: path.join(dir, out), omitBackground: true, clip: { x: 0, y: 0, width: px, height: px } });
    console.log(out, px);
  }
  await b.close();
})();
