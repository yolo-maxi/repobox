import Link from "next/link";
import { LandingHero } from "@/components/landing/Hero";
import { AgentHeatmap } from "@/components/landing/AgentHeatmap";
import { LandingProjects } from "@/components/landing/Projects";
import { LandingShippedToday } from "@/components/landing/ShippedToday";
import { LandingCaseStudies } from "@/components/landing/CaseStudies";
import { LandingEthos } from "@/components/landing/Ethos";
import { LandingCTA } from "@/components/landing/CTA";
import { LandingWriting } from "@/components/landing/Writing";
import { LandingFooter } from "@/components/landing/Footer";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function Home() {
  return (
    <>
      <RegMarks />
      <div
        style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
      >
        <LandingHero />
        <div style={{ height: 80 }} />
        <AgentHeatmap />
        <div style={{ height: 80 }} />
        <LandingProjects />
        <LandingShippedToday />
        <LandingCaseStudies />
        <LandingEthos />
        <LandingCTA />
        <LandingWriting />
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
