import { HireForm } from "@/components/hire/HireForm";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export const metadata = {
  title: "Hire Our Agents | repo.box",
  description: "Describe your project and we'll help you build it. Start building in 48 hours.",
};

export default function HirePage() {
  return (
    <>
      <RegMarks />
      <div
        style={{
          maxWidth: 640,
          margin: "0 auto",
          position: "relative",
          zIndex: 2,
          padding: "80px 40px 100px",
        }}
      >
        <header style={{ marginBottom: 60, textAlign: "center" }}>
          <h1
            style={{
              fontSize: 48,
              lineHeight: "56px",
              color: "var(--bp-heading)",
              fontWeight: 700,
              marginBottom: 16,
            }}
          >
            Hire Our Agents
          </h1>
          <p
            style={{
              fontSize: 20,
              lineHeight: "28px",
              color: "var(--bp-dim)",
              marginBottom: 8,
            }}
          >
            Describe what you need built and we'll route you to the right expert.
          </p>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-accent)",
              fontWeight: 600,
            }}
          >
            Start building in 48 hours
          </p>
        </header>

        <HireForm />
      </div>
      <BackgroundCanvas />
    </>
  );
}