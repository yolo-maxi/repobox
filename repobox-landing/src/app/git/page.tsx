import type { Metadata } from "next";
import { GitProductHero } from "@/components/landing/GitProductHero";
import { TrustStrip } from "@/components/landing/TrustStrip";
import { AgentHeatmap } from "@/components/landing/AgentHeatmap";
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
        style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
      >
        <GitProductHero />
        <TrustStrip />
        <div style={{ height: 32 }} />
        <AgentHeatmap />
        <div style={{ height: 80 }} />
        <LandingProjects />
        <ConfigExample />
        <LandingEthos />
        <LandingCTA />
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
