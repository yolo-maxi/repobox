import { LandingHero } from "@/components/landing/Hero";
import { LandingWork } from "@/components/landing/Work";
import { LandingHireCTA } from "@/components/landing/HireCTA";
import { LandingWriting } from "@/components/landing/Writing";
import { LandingFooter } from "@/components/landing/Footer";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

// BookSprintCTA is intentionally NOT on the homepage (REPOBOX-HOME-001,
// 2026-09-05). It sold fixed-price sprints with unverifiable social proof
// ("13 projects shipped", "2-week average delivery") and routed to a Calendly
// link that returns 404, which contradicted the studio positioning in the hero.
// The component is preserved in src/components/landing/ rather than deleted.

export default function Home() {
  return (
    <>
      <RegMarks />
      <div
        style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
      >
        <LandingHero />
        <LandingWork />
        <LandingHireCTA />
        <LandingWriting />
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
