import { LandingHero } from "@/components/landing/Hero";
import { LandingProjects } from "@/components/landing/Projects";

import { LandingEthos } from "@/components/landing/Ethos";
import { ConfigExample } from "@/components/landing/ConfigExample";
import { LandingCTA } from "@/components/landing/CTA";
import { LandingFooter } from "@/components/landing/Footer";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function Home() {
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
        </div>
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}