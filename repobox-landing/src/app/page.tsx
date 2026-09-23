import { LandingHero } from "@/components/landing/Hero";
import { LandingFocus } from "@/components/landing/Focus";
import { LandingValues } from "@/components/landing/Values";
import { LandingRepositories } from "@/components/landing/Repositories";
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
        <LandingFocus />
        <LandingValues />
        <LandingRepositories />
      </div>
      <LandingFooter />
      <BackgroundCanvas />
    </>
  );
}
