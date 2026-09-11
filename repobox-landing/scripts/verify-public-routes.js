/**
 * verify-public-routes.js — ad-hoc verification, NOT a test suite.
 *
 * This repository has no test runner. These are targeted assertions over the
 * public routes' two standing honesty rules, established by fi_7de1dee1e58ae905b638:
 *
 *   1. No dead host appears as a link target. Retired work is labelled retired,
 *      not linked. Prose may name a dead host (a retirement note has to), so the
 *      assertion is scoped to href/src attributes rather than the bare hostname.
 *   2. No figure appears that cannot be traced to a source. The listed strings
 *      were removed rather than restated with invented provenance.
 *
 * Run against the PACKAGED standalone tree, not the dev server:
 *   pnpm build && node scripts/package-standalone.sh   # or the usual build path
 *   (cd .next/standalone && PORT=3495 node server.js) &
 *   node scripts/verify-public-routes.js
 *
 * Exits non-zero on any failure, so it can be mutation-tested. It is only
 * meaningful if you have watched it go red: change a guard, confirm the right
 * named check fails, restore, confirm green.
 *
 * Playwright is not a dependency of this package. Point PLAYWRIGHT_PATH at an
 * installation if the bare require cannot resolve one.
 */
function loadChromium() {
  const override = process.env.PLAYWRIGHT_PATH;
  for (const spec of [override, "playwright", "/home/xiko/nomad-calendar/node_modules/playwright"]) {
    if (!spec) continue;
    try {
      return require(spec).chromium;
    } catch { /* try next */ }
  }
  console.error("playwright not found; set PLAYWRIGHT_PATH to an installation");
  process.exit(2);
}
const chromium = loadChromium();

const B = process.env.BASE || "http://127.0.0.1:3495";
const ROUTES = ["/projects", "/proof"];
// Folded into /projects on 2026-09-11: each must answer 308 -> /projects.
const REDIRECTS = ["/portfolio", "/building", "/repos", "/projects/supstrategy"];
// The registry both the homepage and /projects render. Read from source so the
// harness follows the list instead of restating it.
const REGISTRY = require("fs").readFileSync(require("path").join(__dirname, "..", "src", "data", "projects.ts"), "utf8");
const PROJECT_NAMES = [...REGISTRY.matchAll(/^\s+name: "([^"]+)",$/gm)].map((m) => m[1]);
const PROJECT_LINKS = [...REGISTRY.matchAll(/href: "(https?:[^"]+)"/g)].map((m) => m[1]);
const VIEWPORTS = [{ width: 1440, height: 900 }, { width: 390, height: 844 }];

// Hosts verified dead by live sweep on 2026-09-06. None may appear as a link
// target. They MAY appear in prose (a retirement note names the host), so the
// assertion is scoped to href/src attributes, not the bare hostname.
const DEAD = ["sss.repo.box", "archipelago.repo.box", "supstrategy.repo.box",
              "rikai.repo.box", "cabin.ai", "itemName=ocean.oceangram"];

// Figures previously shown with no traceable source. Removed, not restated.
const UNSOURCED = ["127 verified", "2.4k", "94% success", "$12.4k", "12.4k",
                   "18% avg", "850+", "1.2k", "50+ active", "76 VS Code",
                   "500+ airlines"];

const fails = [];
let count = 0;
const check = (name, ok) => (count++, ok || fails.push(name));

(async () => {
  const browser = await chromium.launch();
  const html = {};

  for (const vp of VIEWPORTS) {
    const ctx = await browser.newContext({ viewport: vp });
    const tag = `${vp.width}x${vp.height}`;
    for (const route of ROUTES) {
      const page = await ctx.newPage();
      const errors = [], failed = [];
      page.on("pageerror", (e) => errors.push(String(e)));
      page.on("requestfailed", (r) => failed.push(r.url()));
      const resp = await page.goto(B + route, { waitUntil: "networkidle", timeout: 45000 });

      check(`${tag} ${route}: status 200`, resp && resp.status() === 200);
      check(`${tag} ${route}: no JS errors`, errors.length === 0);
      // /concierge/* is served by Caddy in production (proxied to the
      // Concierge server), never by this Next app, so against the bare
      // packaged tree that one request is expected to fail.
      check(`${tag} ${route}: no failed requests`,
        failed.filter((u) => !u.includes("favicon") && !/\/concierge\//.test(u)).length === 0);
      check(`${tag} ${route}: no horizontal overflow`,
        !(await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 1)));
      check(`${tag} ${route}: no zero-size text leaf`,
        (await page.evaluate(() => {
          const bad = [];
          document.querySelectorAll("a,span,div,p,h1,h2,h3,li").forEach((el) => {
            const t = (el.innerText || "").trim();
            if (!t || el.children.length) return;
            const r = el.getBoundingClientRect();
            if (!r.width || !r.height) bad.push(t.slice(0, 40));
          });
          return bad;
        })).length === 0);

      // Dead hosts must not be reachable targets in the live DOM.
      const domDead = await page.evaluate((dead) =>
        [...document.querySelectorAll("a[href],iframe[src],img[src]")]
          .map((e) => e.getAttribute("href") || e.getAttribute("src"))
          .filter((u) => u && dead.some((d) => u.includes(d))), DEAD);
      check(`${tag} ${route}: no dead host in DOM (${domDead.join()})`, domDead.length === 0);

      if (tag === "1440x900") html[route] = await page.content();
      await page.close();
    }
    await ctx.close();
  }
  await browser.close();

  // Served-markup assertions (attribute-scoped, so prose mentions are allowed).
  for (const [route, h] of Object.entries(html)) {
    for (const d of DEAD) {
      const re = new RegExp(`(?:href|src)="[^"]*${d.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`);
      check(`${route}: no link to ${d}`, !re.test(h));
    }
    for (const u of UNSOURCED) check(`${route}: no unsourced '${u}'`, !h.includes(u));
  }

  // /projects is a curated list of current work: no retired/paused/concept
  // sections, fewer than ten entries, every registry entry present, and
  // every external registry link actually rendered as a link target.
  const P = html["/projects"];
  check("registry: parsed at least one project", PROJECT_NAMES.length > 0);
  check(`registry: fewer than ten projects (${PROJECT_NAMES.length})`, PROJECT_NAMES.length < 10);
  for (const w of ["Retired", "Paused", "Concept", "Kanban", "Total Projects"])
    check(`/projects: no '${w}'`, !P.includes(w));
  for (const n of PROJECT_NAMES) check(`/projects: lists ${n}`, P.includes(`>${n}<`));
  for (const l of PROJECT_LINKS) check(`/projects: links ${l}`, P.includes(`href="${l}"`));
  check("/proof: states retirement", /retired/i.test(html["/proof"]));

  // The folded routes must redirect permanently to /projects, not 200 or 404.
  const rctx = await (await chromium.launch()).newContext();
  for (const r of REDIRECTS) {
    const res = await rctx.request.get(B + r, { maxRedirects: 0 });
    const loc = res.headers()["location"] || "";
    check(`${r}: 308 -> /projects (${res.status()} ${loc})`,
      res.status() === 308 && /\/projects$/.test(loc));
  }
  await rctx.browser().close();

  console.log(`${count - fails.length}/${count} ad-hoc assertions passed`);
  fails.forEach((f) => console.log("  FAIL:", f));
  process.exit(fails.length ? 1 : 0);
})();
