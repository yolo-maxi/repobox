"use client";

import { useState } from "react";

interface FormData {
  description: string;
  projectType: string;
  budget: string;
  timeline: string;
  email: string;
}

interface FormErrors {
  description?: string;
  projectType?: string;
  timeline?: string;
  email?: string;
}

const PROJECT_TYPES = [
  "AI Agent",
  "Web App", 
  "Automation",
  "Trading/Finance",
  "Other"
];

const BUDGET_OPTIONS = [
  "Prototype ($1k-5k)",
  "Production ($5k-25k)", 
  "Enterprise ($25k+)",
  "Not sure"
];

const TIMELINE_OPTIONS = [
  "ASAP",
  "2-4 weeks",
  "Flexible"
];

export function HireForm() {
  const [formData, setFormData] = useState<FormData>({
    description: "",
    projectType: "",
    budget: "",
    timeline: "",
    email: "",
  });

  const [errors, setErrors] = useState<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);

  const validateEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    if (!formData.description.trim()) {
      newErrors.description = "Please describe what you need built";
    } else if (formData.description.length > 500) {
      newErrors.description = "Description must be 500 characters or less";
    }

    if (!formData.projectType) {
      newErrors.projectType = "Please select a project type";
    }

    if (!formData.timeline) {
      newErrors.timeline = "Please select a timeline";
    }

    if (!formData.email.trim()) {
      newErrors.email = "Email is required";
    } else if (!validateEmail(formData.email)) {
      newErrors.email = "Please enter a valid email address";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) return;
    
    setIsSubmitting(true);
    
    try {
      const response = await fetch('/api/hire', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(formData),
      });

      if (!response.ok) {
        throw new Error('Failed to submit form');
      }

      setSubmitted(true);
    } catch (error) {
      console.error('Form submission error:', error);
      alert('There was an error submitting your request. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleInputChange = (field: keyof FormData, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    // Clear error when user starts typing
    if (field in errors && errors[field as keyof FormErrors]) {
      setErrors(prev => ({ ...prev, [field as keyof FormErrors]: undefined }));
    }
  };

  if (submitted) {
    return (
      <div
        style={{
          background: "rgba(6, 14, 26, 0.85)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "40px 36px",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          textAlign: "center",
        }}
      >
        <div style={{ fontSize: 48, marginBottom: 16 }}>✅</div>
        <h2
          style={{
            fontSize: 24,
            lineHeight: "32px",
            color: "var(--bp-heading)",
            fontWeight: 600,
            marginBottom: 16,
          }}
        >
          Request Received!
        </h2>
        <p
          style={{
            fontSize: 16,
            lineHeight: "24px",
            color: "var(--bp-dim)",
            marginBottom: 24,
          }}
        >
          We&apos;ll read it and reply to the email you gave us. It goes straight
          to a person, not a queue, so expect a real answer rather than an
          instant autoresponse.
        </p>
        <button
          onClick={() => {
            setSubmitted(false);
            setFormData({
              description: "",
              projectType: "",
              budget: "",
              timeline: "",
              email: "",
            });
          }}
          style={{
            background: "transparent",
            border: "1px solid var(--bp-border)",
            color: "var(--bp-accent)",
            padding: "12px 24px",
            borderRadius: 6,
            cursor: "pointer",
            fontSize: 14,
          }}
        >
          Submit Another Request
        </button>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit}>
      <div
        style={{
          background: "rgba(6, 14, 26, 0.85)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "40px 36px",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
        }}
      >
        {/* Description Field */}
        <div style={{ marginBottom: 24 }}>
          <label
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
            value={formData.description}
            onChange={(e) => handleInputChange("description", e.target.value)}
            placeholder="Describe your project idea, goals, and any specific requirements..."
            style={{
              width: "100%",
              minHeight: 120,
              padding: "12px 16px",
              background: "rgba(0, 0, 0, 0.3)",
              border: errors.description ? "1px solid #ff4444" : "1px solid var(--bp-border)",
              borderRadius: 6,
              color: "var(--bp-heading)",
              fontSize: 14,
              lineHeight: "20px",
              resize: "vertical",
            }}
            maxLength={500}
          />
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginTop: 8,
            }}
          >
            {errors.description && (
              <span style={{ color: "#ff4444", fontSize: 12 }}>{errors.description}</span>
            )}
            <span
              style={{
                color: formData.description.length > 450 ? "#ff4444" : "var(--bp-dim)",
                fontSize: 12,
                marginLeft: "auto",
              }}
            >
              {formData.description.length}/500
            </span>
          </div>
        </div>

        {/* Project Type */}
        <div style={{ marginBottom: 24 }}>
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
          <select
            value={formData.projectType}
            onChange={(e) => handleInputChange("projectType", e.target.value)}
            style={{
              width: "100%",
              padding: "12px 16px",
              background: "rgba(0, 0, 0, 0.3)",
              border: errors.projectType ? "1px solid #ff4444" : "1px solid var(--bp-border)",
              borderRadius: 6,
              color: "var(--bp-heading)",
              fontSize: 14,
            }}
          >
            <option value="">Select project type</option>
            {PROJECT_TYPES.map(type => (
              <option key={type} value={type}>{type}</option>
            ))}
          </select>
          {errors.projectType && (
            <span style={{ color: "#ff4444", fontSize: 12, marginTop: 4, display: "block" }}>
              {errors.projectType}
            </span>
          )}
        </div>

        {/* Budget */}
        <div style={{ marginBottom: 24 }}>
          <label
            style={{
              display: "block",
              fontSize: 14,
              fontWeight: 600,
              color: "var(--bp-heading)",
              marginBottom: 8,
            }}
          >
            Budget Expectation
          </label>
          <select
            value={formData.budget}
            onChange={(e) => handleInputChange("budget", e.target.value)}
            style={{
              width: "100%",
              padding: "12px 16px",
              background: "rgba(0, 0, 0, 0.3)",
              border: "1px solid var(--bp-border)",
              borderRadius: 6,
              color: "var(--bp-heading)",
              fontSize: 14,
            }}
          >
            <option value="">Select budget range</option>
            {BUDGET_OPTIONS.map(budget => (
              <option key={budget} value={budget}>{budget}</option>
            ))}
          </select>
        </div>

        {/* Timeline */}
        <div style={{ marginBottom: 24 }}>
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
          <select
            value={formData.timeline}
            onChange={(e) => handleInputChange("timeline", e.target.value)}
            style={{
              width: "100%",
              padding: "12px 16px",
              background: "rgba(0, 0, 0, 0.3)",
              border: errors.timeline ? "1px solid #ff4444" : "1px solid var(--bp-border)",
              borderRadius: 6,
              color: "var(--bp-heading)",
              fontSize: 14,
            }}
          >
            <option value="">Select timeline</option>
            {TIMELINE_OPTIONS.map(timeline => (
              <option key={timeline} value={timeline}>{timeline}</option>
            ))}
          </select>
          {errors.timeline && (
            <span style={{ color: "#ff4444", fontSize: 12, marginTop: 4, display: "block" }}>
              {errors.timeline}
            </span>
          )}
        </div>

        {/* Email */}
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
            Email for follow-up *
          </label>
          <input
            type="email"
            value={formData.email}
            onChange={(e) => handleInputChange("email", e.target.value)}
            placeholder="your@email.com"
            style={{
              width: "100%",
              padding: "12px 16px",
              background: "rgba(0, 0, 0, 0.3)",
              border: errors.email ? "1px solid #ff4444" : "1px solid var(--bp-border)",
              borderRadius: 6,
              color: "var(--bp-heading)",
              fontSize: 14,
            }}
          />
          {errors.email && (
            <span style={{ color: "#ff4444", fontSize: 12, marginTop: 4, display: "block" }}>
              {errors.email}
            </span>
          )}
        </div>

        {/* Submit Button */}
        <button
          type="submit"
          disabled={isSubmitting}
          style={{
            width: "100%",
            padding: "16px 32px",
            background: isSubmitting ? "var(--bp-border)" : "var(--bp-accent)",
            color: "#ffffff",
            border: "none",
            borderRadius: 8,
            fontSize: 16,
            fontWeight: 600,
            cursor: isSubmitting ? "not-allowed" : "pointer",
            transition: "background 0.2s",
          }}
        >
          {isSubmitting ? "Processing..." : "Submit Request"}
        </button>

        {/* Footer Note */}
        <p
          style={{
            fontSize: 12,
            color: "var(--bp-dim)",
            textAlign: "center",
            marginTop: 16,
            lineHeight: "16px",
          }}
        >
          Goes straight to us. We read every one and reply by email, usually
          within a day.
        </p>
      </div>
    </form>
  );
}
