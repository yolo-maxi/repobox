import { LandingHero } from "@/components/landing/Hero";
import { BookSprintCTA } from "@/components/landing/BookSprintCTA";
import { LandingHireCTA } from "@/components/landing/HireCTA";
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
        <BookSprintCTA />
        <div style={{ height: 32 }} />
        <LandingHireCTA />
        <LandingWriting />
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
