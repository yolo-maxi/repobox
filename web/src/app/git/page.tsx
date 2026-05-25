import type { Metadata } from "next";
import { GitProductHero } from "@/components/landing/GitProductHero";
import { LandingProjects } from "@/components/landing/Projects";
import { ConfigExample } from "@/components/landing/ConfigExample";
import { LandingEthos } from "@/components/landing/Ethos";
import { LandingCTA } from "@/components/landing/CTA";
import { LandingFooter } from "@/components/landing/Footer";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export const metadata: Metadata = {
  title: "repo.box git layer",
  description: "A Git permission layer that makes repositories safe for AI agents.",
  openGraph: {
    title: "repo.box git layer",
    description: "A Git permission layer that makes repositories safe for AI agents.",
    url: "https://repo.box/git",
    siteName: "repo.box",
    images: [{ url: "/og/repo-box-share.jpg", width: 1200, height: 630 }],
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "repo.box git layer",
    description: "A Git permission layer that makes repositories safe for AI agents.",
    images: ["/og/repo-box-share.jpg"],
  },
};

export default function GitPage() {
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
          <GitProductHero />
          <div style={{ height: "clamp(20px, 5vw, 32px)" }} />
          <LandingProjects />
          <ConfigExample />
          <LandingEthos />
          <LandingCTA />
        </div>
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
