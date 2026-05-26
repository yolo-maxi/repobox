import type { Metadata } from "next";
import Link from "next/link";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";
import { RegMarks } from "@/components/RegMarks";
import { SiteNav } from "@/components/SiteNav";

const proofs = [
  {
    title: "Homepage became product-first again",
    date: "2026-05-26",
    commit: "94fdc267",
    artifact: "/blog/homepage-is-not-the-product.html",
    result: "Published a positioning essay with HTML, plain-text, and RSS coverage.",
    check: "Blog index, post, text version, and feed verified publicly.",
  },
  {
    title: "No more dead-end header clicks",
    date: "2026-05-26",
    commit: "5842bb6e",
    artifact: "/",
    result: "Removed empty homepage nav targets so the first impression only points at live routes.",
    check: "Header routes returned HTTP 200 after rebuild.",
  },
  {
    title: "Git layer has a real home",
    date: "2026-05-25",
    commit: "6c084ef6",
    artifact: "/git",
    result: "Moved Git permission-layer content out of the studio homepage and into a focused product page.",
    check: "Middleware reserves /git so it no longer rewrites to the repo explorer.",
  },
  {
    title: "Merge safety got config validation",
    date: "2026-03-30",
    commit: "3fd239cb",
    artifact: "/docs/spec/permissions",
    result: "Added validation work for safer agent merge permissions.",
    check: "Implementation landed as REPO-043 with repo-local docs and checks.",
  },
  {
    title: "Portfolio wall became evidence-led",
    date: "2026-03-30",
    commit: "00559d94",
    artifact: "/projects",
    result: "Live portfolio cards now point visitors at actual shipped work instead of abstract claims.",
    check: "Implemented as REPO-042 and exposed on the public projects route.",
  },
];

export const metadata: Metadata = {
  title: "Proofs — repo.box",
  description: "Recent repo.box ships with commits, checks, and public artifacts.",
  openGraph: {
    title: "Proofs — repo.box",
    description: "Recent repo.box ships with commits, checks, and public artifacts.",
    url: "https://repo.box/proofs",
    siteName: "repo.box",
    images: [{ url: "/og/repo-box-share.jpg", width: 1200, height: 630 }],
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Proofs — repo.box",
    description: "Recent repo.box ships with commits, checks, and public artifacts.",
    images: ["/og/repo-box-share.jpg"],
  },
};

function ProofCard({ proof, index }: { proof: (typeof proofs)[number]; index: number }) {
  return (
    <article className="project-card" style={{
      position: "relative",
      overflow: "hidden",
      background: "rgba(13, 31, 53, 0.82)",
      border: "1px solid var(--bp-border)",
      borderRadius: 12,
      padding: "22px",
    }}>
      <svg className="card-border"><rect x="0.5" y="0.5" width="calc(100% - 1px)" height="calc(100% - 1px)" /></svg>
      <div style={{ position: "relative", zIndex: 2 }}>
        <div style={{ display: "flex", justifyContent: "space-between", gap: 16, alignItems: "flex-start", marginBottom: 14 }}>
          <div>
            <div style={{ color: "var(--bp-accent)", fontSize: 11, letterSpacing: "0.14em", textTransform: "uppercase", marginBottom: 8 }}>
              proof {String(index + 1).padStart(2, "0")} · {proof.date}
            </div>
            <h2 style={{ color: "var(--bp-heading)", fontSize: 20, lineHeight: "28px", margin: 0 }}>{proof.title}</h2>
          </div>
          <code style={{ color: "var(--bp-accent2)", background: "rgba(79,195,247,0.08)", border: "1px solid rgba(79,195,247,0.18)", borderRadius: 999, padding: "5px 10px", whiteSpace: "nowrap" }}>
            {proof.commit}
          </code>
        </div>
        <div style={{ display: "grid", gap: 12 }}>
          <p style={{ color: "var(--bp-text)", fontSize: 13, lineHeight: "22px", margin: 0 }}>{proof.result}</p>
          <p style={{ color: "var(--bp-dim)", fontSize: 12, lineHeight: "20px", margin: 0 }}>Check: {proof.check}</p>
          <Link href={proof.artifact} style={{ color: "var(--bp-accent)", fontSize: 12, fontWeight: 700 }}>
            View artifact →
          </Link>
        </div>
      </div>
    </article>
  );
}

export default function ProofsPage() {
  return (
    <>
      <RegMarks />
      <SiteNav />
      <main style={{ maxWidth: 920, margin: "0 auto", position: "relative", zIndex: 2, padding: "120px clamp(20px, 5vw, 40px) 100px" }}>
        <section style={{ marginBottom: 36 }}>
          <div style={{ color: "var(--bp-accent)", fontSize: 12, letterSpacing: "0.18em", textTransform: "uppercase", marginBottom: 18 }}>
            receipts, not vibes
          </div>
          <h1 style={{ color: "var(--bp-heading)", fontSize: "clamp(38px, 8vw, 72px)", lineHeight: 1, letterSpacing: "-0.06em", marginBottom: 22 }}>
            Proofs of ship.
          </h1>
          <p style={{ color: "var(--bp-dim)", fontSize: 16, lineHeight: "28px", maxWidth: 650 }}>
            A compact public ledger of recent repo.box work: what changed, where the artifact lives, and how it was checked.
          </p>
        </section>

        <section style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))", gap: 16, marginBottom: 40 }}>
          <div style={{ background: "rgba(79,195,247,0.08)", border: "1px solid rgba(79,195,247,0.2)", borderRadius: 12, padding: 18 }}>
            <div style={{ color: "var(--bp-heading)", fontSize: 28, fontWeight: 800 }}>5</div>
            <div style={{ color: "var(--bp-dim)", fontSize: 12 }}>recent public proofs</div>
          </div>
          <div style={{ background: "rgba(94,253,245,0.08)", border: "1px solid rgba(94,253,245,0.18)", borderRadius: 12, padding: 18 }}>
            <div style={{ color: "var(--bp-heading)", fontSize: 28, fontWeight: 800 }}>100%</div>
            <div style={{ color: "var(--bp-dim)", fontSize: 12 }}>linked to artifacts</div>
          </div>
          <div style={{ background: "rgba(13,31,53,0.82)", border: "1px solid var(--bp-border)", borderRadius: 12, padding: 18 }}>
            <div style={{ color: "var(--bp-heading)", fontSize: 28, fontWeight: 800 }}>0</div>
            <div style={{ color: "var(--bp-dim)", fontSize: 12 }}>claims without checks</div>
          </div>
        </section>

        <section style={{ display: "grid", gap: 16 }}>
          {proofs.map((proof, index) => <ProofCard key={proof.commit} proof={proof} index={index} />)}
        </section>
      </main>
      <BackgroundCanvas />
    </>
  );
}
