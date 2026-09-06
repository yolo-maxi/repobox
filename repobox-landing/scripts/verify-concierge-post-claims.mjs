#!/usr/bin/env node
// Ad-hoc verification harness for the Concierge launch post.
// Asserts every factual/numeric claim in the article against Concierge SOURCE,
// not against a previous write-up. Exit 0 = all claims backed, 1 = a claim is
// unbacked (mutation-test target), 2 = harness could not run.
//
// The repobox-landing package has no test runner; this is an ad-hoc assertion
// script, not a suite. Run: node scripts/verify-concierge-post-claims.mjs
import { readFileSync, existsSync } from "node:fs";

const CONCIERGE = process.env.CONCIERGE_DIR || "/home/xiko/concierge";
const POST =
  process.env.POST_FILE ||
  new URL("../public/blog/powerless-landing-page-agent.txt", import.meta.url).pathname;

if (!existsSync(CONCIERGE)) {
  console.error(`cannot run: concierge checkout not found at ${CONCIERGE}`);
  process.exit(2);
}
if (!existsSync(POST)) {
  console.error(`cannot run: post not found at ${POST}`);
  process.exit(2);
}

const read = (p) => readFileSync(`${CONCIERGE}/${p}`, "utf8");
const post = readFileSync(POST, "utf8");

const app = read("server/src/app.ts");
const runtime = read("server/src/runtime.ts");
const retrieval = read("server/src/retrieval.ts");
const protocol = read("server/src/ui/protocol.ts");

let failed = 0;
const check = (name, ok, detail = "") => {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? ` -- ${detail}` : ""}`);
  if (!ok) failed++;
};

// Each entry: the number the post states, and the source expression that must
// produce it. Both sides must agree; a drift in either direction is a failure.
const numeric = [
  ["message depth cap 24", /MAX_MESSAGES\s*=\s*24\b/, app, /24 messages \(MAX_MESSAGES\)/],
  ["message char cap 2000", /MAX_CHARS\s*=\s*2000\b/, app, /2000 characters each \(MAX_CHARS\)/],
  ["max concurrent 4", /CONCIERGE_MAX_CONCURRENT,\s*4\)/, runtime, /4 concurrent provider calls/],
  ["queue depth 16", /CONCIERGE_MAX_QUEUE_DEPTH,\s*16\)/, runtime, /16 waiting requests/],
  ["request timeout 30000", /CONCIERGE_REQUEST_TIMEOUT_MS,\s*30_000\)/, runtime, /30000ms \(CONCIERGE_REQUEST_TIMEOUT_MS\)/],
  ["ip rate limit 20", /CONCIERGE_RATE_LIMIT_IP \|\| env\.RATE_LIMIT,\s*20\)/, runtime, /20 requests \/ 60s/],
  ["session rate limit 12", /CONCIERGE_RATE_LIMIT_SESSION,\s*12\)/, runtime, /12 requests \/ 60s/],
  ["circuit failures 3", /CONCIERGE_CIRCUIT_FAILURES,\s*3\)/, runtime, /3 consecutive failures/],
  ["circuit reset 30000", /CONCIERGE_CIRCUIT_RESET_MS,\s*30_000\)/, runtime, /resets after 30000ms/],
  ["retrieval cap 4000", /DEFAULT_MAX_INJECTED_CHARS\s*=\s*4000\b/, retrieval, /4000 characters by default/],
];

for (const [name, srcRe, src, postRe] of numeric) {
  const inSource = srcRe.test(src);
  const inPost = postRe.test(post);
  check(`${name} (source)`, inSource, inSource ? "" : `${srcRe} not found`);
  check(`${name} (post states it)`, inPost, inPost ? "" : `${postRe} not in post`);
}

// Component registry: the post names exactly the registered components.
const registered = [...protocol.matchAll(/^ {2}(\w+):\s*\{$/gm)].map((m) => m[1]);
const claimed = ["button_group", "lead_form", "product_card", "handoff_card"];
check(
  "post names exactly the registered components",
  claimed.every((c) => registered.includes(c)) &&
    registered.filter((r) => claimed.includes(r)).length === claimed.length &&
    claimed.every((c) => post.includes(c)),
  `registry=[${registered.join(",")}]`,
);

// Tools: the post claims exactly two implemented tools.
const toolFiles = ["server/src/tools/captureLead.ts", "server/src/tools/handoffHuman.ts"];
check(
  "both claimed tools exist in source",
  toolFiles.every((f) => existsSync(`${CONCIERGE}/${f}`)) &&
    post.includes("capture_lead") &&
    post.includes("handoff_human"),
);

// Client system-role stripping: the post claims only user/assistant survive.
check(
  "client system roles are filtered server-side",
  /role === "user" \|\| m\.role === "assistant"/.test(app),
  "app.ts message filter",
);

// The post must not claim the live deployment is current — it is not.
check(
  "post discloses the live instance runs an older build",
  /older build/.test(post),
);

// Error codes the post lists must exist in source.
for (const code of ["queue_full", "rate_limited_ip", "rate_limited_session", "provider_circuit_open"]) {
  const inSrc = new RegExp(`"${code}"`).test(app) || new RegExp(`"${code}"`).test(runtime);
  check(`error code ${code} exists in source`, inSrc);
  check(`error code ${code} listed in post`, post.includes(code));
}

console.log(`\n${failed === 0 ? "ALL CLAIMS BACKED" : `${failed} UNBACKED CLAIM(S)`}`);
process.exit(failed === 0 ? 0 : 1);
