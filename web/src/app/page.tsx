import { LandingHero } from "@/components/landing/Hero";
import { LandingProjects } from "@/components/landing/Projects";
import { LandingEthos } from "@/components/landing/Ethos";
import { ConfigExample } from "@/components/landing/ConfigExample";
import { LandingCTA } from "@/components/landing/CTA";
import { LandingFooter } from "@/components/landing/Footer";
import { LiveFeed } from "@/components/landing/LiveFeed";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";
import { execSync } from "child_process";

function getRecentPushes() {
  try {
    const sql = [
      "SELECT substr(commit_hash, 1, 7) AS hash,",
      "COALESCE(NULLIF(pusher_address, ''), address) AS signer,",
      "commit_message, pushed_at",
      "FROM push_log",
      "WHERE name='wall'",
      "ORDER BY pushed_at DESC",
      "LIMIT 5;",
    ].join(' ');

    const raw = execSync(
      `sqlite3 -separator '|' /var/lib/repobox/repos/repobox.db \"${sql}\"`,
      { timeout: 3000, encoding: "utf8" }
    );

    return raw
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line) => {
        const [hash, author, message, pushedAt] = line.split('|');
        return {
          hash,
          author,
          message,
          time: new Date(Number(pushedAt) * 1000).toISOString(),
        };
      });
  } catch {
    return [];
  }
}

export const revalidate = 60; // revalidate every 60 seconds

export default function Home() {
  const entries = getRecentPushes();

  return (
    <>
      <RegMarks />
      <div
        style={{
          maxWidth: 750,
          margin: "0 auto",
          position: "relative",
          zIndex: 2,
          padding: "clamp(20px, 8vw, 80px) clamp(20px, 5vw, 40px) 100px"
        }}
      >
        {/* Frosted glass content column */}
        <div
          style={{
            background: "rgba(6, 14, 26, 0.75)",
            backdropFilter: "blur(16px)",
            WebkitBackdropFilter: "blur(16px)",
            border: "1px solid rgba(79, 195, 247, 0.1)",
            borderRadius: 12,
            padding: "clamp(20px, 5vw, 40px)",
          }}
        >
          <LandingHero />
          <div style={{ height: "clamp(20px, 5vw, 32px)" }} />
          <LandingProjects />
          <ConfigExample />
          <LandingEthos />
          <LandingCTA />
          <LiveFeed entries={entries} />
        </div>
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
