import Link from "next/link";
import { LandingHero } from "@/components/landing/Hero";
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
        style={{ 
          maxWidth: 720, 
          margin: "0 auto", 
          position: "relative", 
          zIndex: 2, 
          padding: "clamp(20px, 8vw, 80px) clamp(20px, 5vw, 40px) 100px" 
        }}
      >
        <LandingHero />
        <div style={{ height: "clamp(40px, 10vw, 80px)" }} />
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