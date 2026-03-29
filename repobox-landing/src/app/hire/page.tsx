"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function HirePage() {
  const pageRef = useRef<HTMLElement>(null);
  const [formData, setFormData] = useState({
    description: "",
    projectType: "",
    budget: "",
    email: "",
    timeline: "",
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) e.target.classList.add("visible");
        });
      },
      { threshold: 0.1 }
    );
    if (pageRef.current) observer.observe(pageRef.current);
    return () => observer.disconnect();
  }, []);

  const validateForm = () => {
    const newErrors: Record<string, string> = {};
    
    if (!formData.description.trim()) {
      newErrors.description = "Please describe what you need built";
    } else if (formData.description.length > 500) {
      newErrors.description = "Description must be under 500 characters";
    }
    
    if (!formData.projectType) {
      newErrors.projectType = "Please select a project type";
    }
    
    if (!formData.email.trim()) {
      newErrors.email = "Email is required for follow-up";
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      newErrors.email = "Please enter a valid email address";
    }
    
    if (!formData.timeline) {
      newErrors.timeline = "Please select your timeline preference";
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const getRouting = () => {
    const { projectType, budget } = formData;
    
    if (projectType === "AI Agent") {
      return { type: "telegram", url: "https://t.me/ocean_king_bot" };
    } else if (projectType === "Automation") {
      return { type: "telegram", url: "https://t.me/ocean_king_bot" };
    } else if (projectType === "Web App" && (budget === "Production ($5k-25k)" || budget === "Enterprise ($25k+)")) {
      return { type: "calendly", url: "https://calendly.com/repo-box/consultation" };
    } else {
      return { type: "telegram", url: "https://t.me/ocean_king_bot" };
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) return;
    
    setIsSubmitting(true);
    
    try {
      // Send form data to backend
      const response = await fetch('/api/hire', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      });
      
      if (response.ok) {
        setSubmitted(true);
        // Analytics event would go here
      } else {
        throw new Error('Submission failed');
      }
    } catch (error) {
      setErrors({ submit: 'Submission failed. Please try again.' });
    } finally {
      setIsSubmitting(false);
    }
  };

  const routing = getRouting();

  if (submitted) {
    return (
      <>
        <RegMarks />
        <div
          style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
        >
          <header style={{ marginBottom: 60 }}>
            <nav
              style={{
                display: "flex",
                gap: 16,
                marginBottom: 24,
                fontSize: 12,
              }}
            >
              <Link
                href="/"
                style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
                onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
                onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
              >
                ← home
              </Link>
              <Link
                href="/projects"
                style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
                onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
                onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
              >
                projects
              </Link>
            </nav>
          </header>

          <main style={{ textAlign: "center", padding: "60px 0" }}>
            <div style={{ fontSize: 48, marginBottom: 24 }}>✅</div>
            <h1
              className="font-mono font-bold"
              style={{ fontSize: 32, lineHeight: 1.2, marginBottom: 20, color: "var(--bp-heading)" }}
            >
              We'll be in touch!
            </h1>
            <p style={{ fontSize: 16, lineHeight: "24px", color: "var(--bp-text)", marginBottom: 40 }}>
              Your request has been received. You should hear back within 24 hours.
            </p>
            
            <div
              style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 32,
                marginBottom: 40,
                textAlign: "left",
              }}
            >
              <h2 style={{ fontSize: 18, marginBottom: 16, color: "var(--bp-heading)" }}>
                Next Steps
              </h2>
              {routing.type === "telegram" ? (
                <p style={{ fontSize: 14, lineHeight: "22px", color: "var(--bp-text)", marginBottom: 20 }}>
                  For fastest response, reach out directly to Ocean on Telegram:
                </p>
              ) : (
                <p style={{ fontSize: 14, lineHeight: "22px", color: "var(--bp-text)", marginBottom: 20 }}>
                  For larger projects, book a consultation call:
                </p>
              )}
              
              <a
                href={routing.url}
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  display: "inline-block",
                  background: "var(--bp-accent)",
                  color: "#0a1628",
                  padding: "12px 24px",
                  borderRadius: 6,
                  textDecoration: "none",
                  fontSize: 14,
                  fontWeight: 600,
                  transition: "all 0.2s",
                }}
                onMouseOver={(e) => {
                  e.currentTarget.style.background = "var(--bp-accent2)";
                  e.currentTarget.style.transform = "translateY(-1px)";
                }}
                onMouseOut={(e) => {
                  e.currentTarget.style.background = "var(--bp-accent)";
                  e.currentTarget.style.transform = "translateY(0)";
                }}
              >
                {routing.type === "telegram" ? "Message Ocean" : "Book Consultation"}
              </a>
            </div>
          </main>
        </div>
        <BackgroundCanvas />
      </>
    );
  }

  return (
    <>
      <RegMarks />
      <div
        style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
      >
        {/* Header */}
        <header style={{ marginBottom: 60 }}>
          <nav
            style={{
              display: "flex",
              gap: 16,
              marginBottom: 24,
              fontSize: 12,
            }}
          >
            <Link
              href="/"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              ← home
            </Link>
            <Link
              href="/projects"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              projects
            </Link>
          </nav>
          <div
            className="font-mono font-bold"
            style={{ fontSize: 36, lineHeight: 1.1, marginBottom: 16 }}
          >
            Let's Build Something
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 500,
            }}
          >
            Tell us what you need built. We'll route you to the right agent or book you a consultation.
          </p>
        </header>

        <main ref={pageRef} className="reveal">
          <form onSubmit={handleSubmit} style={{ maxWidth: 600 }}>
            {/* Project Description */}
            <div style={{ marginBottom: 32 }}>
              <label
                htmlFor="description"
                style={{
                  display: "block",
                  fontSize: 14,
                  fontWeight: 600,
                  color: "var(--bp-heading)",
                  marginBottom: 8,
                }}
              >
                What do you need built? *
              </label>
              <textarea
                id="description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Describe your project idea, challenge, or automation need..."
                style={{
                  width: "100%",
                  minHeight: 120,
                  background: "var(--bp-surface)",
                  border: `1px solid ${errors.description ? "#ef4444" : "var(--bp-border)"}`,
                  borderRadius: 6,
                  padding: 16,
                  fontSize: 14,
                  color: "var(--bp-text)",
                  resize: "vertical",
                  fontFamily: "inherit",
                }}
                maxLength={500}
              />
              <div style={{ display: "flex", justifyContent: "space-between", marginTop: 4 }}>
                <div style={{ fontSize: 12, color: "#ef4444" }}>
                  {errors.description && errors.description}
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)" }}>
                  {formData.description.length}/500
                </div>
              </div>
            </div>

            {/* Project Type */}
            <div style={{ marginBottom: 32 }}>
              <label
                style={{
                  display: "block",
                  fontSize: 14,
                  fontWeight: 600,
                  color: "var(--bp-heading)",
                  marginBottom: 8,
                }}
              >
                Project Type *
              </label>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))", gap: 12 }}>
                {["AI Agent", "Web App", "Automation", "Other"].map((type) => (
                  <label
                    key={type}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                      padding: 12,
                      background: "var(--bp-surface)",
                      border: `1px solid ${errors.projectType ? "#ef4444" : "var(--bp-border)"}`,
                      borderRadius: 6,
                      cursor: "pointer",
                      transition: "border-color 0.2s",
                    }}
                  >
                    <input
                      type="radio"
                      name="projectType"
                      value={type}
                      checked={formData.projectType === type}
                      onChange={(e) => setFormData({ ...formData, projectType: e.target.value })}
                      style={{
                        accentColor: "var(--bp-accent)",
                      }}
                    />
                    <span style={{ fontSize: 14, color: "var(--bp-text)" }}>{type}</span>
                  </label>
                ))}
              </div>
              {errors.projectType && (
                <div style={{ fontSize: 12, color: "#ef4444", marginTop: 4 }}>
                  {errors.projectType}
                </div>
              )}
            </div>

            {/* Budget Range */}
            <div style={{ marginBottom: 32 }}>
              <label
                style={{
                  display: "block",
                  fontSize: 14,
                  fontWeight: 600,
                  color: "var(--bp-heading)",
                  marginBottom: 8,
                }}
              >
                Budget Range
              </label>
              <select
                value={formData.budget}
                onChange={(e) => setFormData({ ...formData, budget: e.target.value })}
                style={{
                  width: "100%",
                  background: "var(--bp-surface)",
                  border: "1px solid var(--bp-border)",
                  borderRadius: 6,
                  padding: 12,
                  fontSize: 14,
                  color: "var(--bp-text)",
                  fontFamily: "inherit",
                }}
              >
                <option value="">Select budget range...</option>
                <option value="Prototype ($1k-5k)">Prototype ($1k-5k)</option>
                <option value="Production ($5k-25k)">Production ($5k-25k)</option>
                <option value="Enterprise ($25k+)">Enterprise ($25k+)</option>
              </select>
            </div>

            {/* Email */}
            <div style={{ marginBottom: 32 }}>
              <label
                htmlFor="email"
                style={{
                  display: "block",
                  fontSize: 14,
                  fontWeight: 600,
                  color: "var(--bp-heading)",
                  marginBottom: 8,
                }}
              >
                Email *
              </label>
              <input
                type="email"
                id="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                placeholder="your@email.com"
                style={{
                  width: "100%",
                  background: "var(--bp-surface)",
                  border: `1px solid ${errors.email ? "#ef4444" : "var(--bp-border)"}`,
                  borderRadius: 6,
                  padding: 12,
                  fontSize: 14,
                  color: "var(--bp-text)",
                  fontFamily: "inherit",
                }}
              />
              {errors.email && (
                <div style={{ fontSize: 12, color: "#ef4444", marginTop: 4 }}>
                  {errors.email}
                </div>
              )}
            </div>

            {/* Timeline */}
            <div style={{ marginBottom: 40 }}>
              <label
                style={{
                  display: "block",
                  fontSize: 14,
                  fontWeight: 600,
                  color: "var(--bp-heading)",
                  marginBottom: 8,
                }}
              >
                Timeline *
              </label>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(120px, 1fr))", gap: 12 }}>
                {["ASAP", "2-4 weeks", "Flexible"].map((timeline) => (
                  <label
                    key={timeline}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                      padding: 12,
                      background: "var(--bp-surface)",
                      border: `1px solid ${errors.timeline ? "#ef4444" : "var(--bp-border)"}`,
                      borderRadius: 6,
                      cursor: "pointer",
                      transition: "border-color 0.2s",
                    }}
                  >
                    <input
                      type="radio"
                      name="timeline"
                      value={timeline}
                      checked={formData.timeline === timeline}
                      onChange={(e) => setFormData({ ...formData, timeline: e.target.value })}
                      style={{
                        accentColor: "var(--bp-accent)",
                      }}
                    />
                    <span style={{ fontSize: 14, color: "var(--bp-text)" }}>{timeline}</span>
                  </label>
                ))}
              </div>
              {errors.timeline && (
                <div style={{ fontSize: 12, color: "#ef4444", marginTop: 4 }}>
                  {errors.timeline}
                </div>
              )}
            </div>

            {/* Submit */}
            <div>
              <button
                type="submit"
                disabled={isSubmitting}
                style={{
                  background: isSubmitting ? "var(--bp-dim)" : "var(--bp-accent)",
                  color: "#0a1628",
                  border: "none",
                  borderRadius: 6,
                  padding: "16px 32px",
                  fontSize: 16,
                  fontWeight: 600,
                  cursor: isSubmitting ? "not-allowed" : "pointer",
                  transition: "all 0.2s",
                  fontFamily: "inherit",
                }}
                onMouseOver={(e) => {
                  if (!isSubmitting) {
                    e.currentTarget.style.background = "var(--bp-accent2)";
                    e.currentTarget.style.transform = "translateY(-1px)";
                  }
                }}
                onMouseOut={(e) => {
                  if (!isSubmitting) {
                    e.currentTarget.style.background = "var(--bp-accent)";
                    e.currentTarget.style.transform = "translateY(0)";
                  }
                }}
              >
                {isSubmitting ? "Submitting..." : "Get Started"}
              </button>
              
              {errors.submit && (
                <div style={{ fontSize: 12, color: "#ef4444", marginTop: 8 }}>
                  {errors.submit}
                </div>
              )}
            </div>
          </form>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}