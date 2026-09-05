"use client";
import { useState } from "react";

type BudgetRange = "under-5k" | "5k-25k" | "25k-plus";
type Timeline = "this-week" | "next-2-weeks" | "flexible";

interface QualificationState {
  budget: BudgetRange | null;
  timeline: Timeline | null;
}

export function BookSprintCTA() {
  const [qualification, setQualification] = useState<QualificationState>({
    budget: null,
    timeline: null,
  });
  const [showResult, setShowResult] = useState(false);

  const handleQualification = (budget: BudgetRange, timeline: Timeline) => {
    setQualification({ budget, timeline });
    setShowResult(true);
  };

  const renderRouting = () => {
    const { budget, timeline } = qualification;
    if (!budget || !timeline) return null;

    // Smart routing logic
    if (budget === "under-5k" && timeline === "flexible") {
      return (
        <div className="routing-result prototype">
          <div className="result-content">
            <div className="icon">🚀</div>
            <h3>Prototype Sprint Perfect Fit!</h3>
            <p>Your requirements match our Prototype Sprint package perfectly. Let's discuss your project.</p>
            <a 
              href="https://t.me/ocean_king_bot?start=prototype-inquiry"
              className="cta-button primary"
            >
              Start Prototype Discussion
            </a>
            <p className="secondary">Response within 2 hours • $1k-5k range</p>
          </div>
        </div>
      );
    }

    if (budget === "5k-25k" && (timeline === "this-week" || timeline === "next-2-weeks")) {
      return (
        <div className="routing-result consultation">
          <div className="result-content">
            <div className="icon">📞</div>
            <h3>Priority Consultation Slot</h3>
            <p>Your timeline needs priority attention. Let's schedule a 30-minute strategy call.</p>
            <a 
              href="https://calendly.com/francesco-repo-box/30min"
              className="cta-button primary"
            >
              Book Strategy Call
            </a>
            <p className="secondary">Next available slot • $5k-25k projects</p>
          </div>
        </div>
      );
    }

    if (budget === "25k-plus") {
      return (
        <div className="routing-result enterprise">
          <div className="result-content">
            <div className="icon">🏢</div>
            <h3>Enterprise Sprint</h3>
            <p>Your project qualifies for our enterprise track with dedicated team allocation.</p>
            <a 
              href="mailto:enterprise@repo.box?subject=Enterprise Sprint Inquiry&body=Budget: $25k+ | Timeline: {timeline} | Priority: High"
              className="cta-button primary"
            >
              Contact Enterprise Team
            </a>
            <p className="secondary">24h response guarantee • Custom solutions</p>
          </div>
        </div>
      );
    }

    // Poor fit combinations - fast decline path
    if ((budget === "under-5k" && timeline !== "flexible") || 
        (budget === "5k-25k" && timeline === "flexible")) {
      return (
        <div className="routing-result decline">
          <div className="result-content">
            <div className="icon">💡</div>
            <h3>Alternative Resources</h3>
            <p>This combination doesn't align with our sprint model. Try these resources instead:</p>
            <div className="alternatives">
              {/* /packages quarantined from homepage navigation by 2026-08-17 disposition. */}
              <a href="/proof" className="alternative-link">Case Studies</a>
              <a href="https://repo.box/blog" className="alternative-link">Build Guides</a>
            </div>
            <p className="secondary">Or contact us for custom arrangements</p>
          </div>
        </div>
      );
    }

    return null;
  };

  const resetQualification = () => {
    setQualification({ budget: null, timeline: null });
    setShowResult(false);
  };

  return (
    <section className="book-sprint-cta">
      {!showResult ? (
        <div className="qualification-flow">
          <div className="header">
            <div className="value-prop">
              <h2>Start Building in 48 Hours</h2>
              <p>Choose your sprint type and timeline for instant routing</p>
              <div className="social-proof">
                <span>13 projects shipped • 2-week average delivery</span>
              </div>
            </div>
          </div>

          <div className="qualification-grid">
            <div className="qual-section">
              <h3>Project Budget</h3>
              <div className="option-group">
                {[
                  { id: "under-5k" as BudgetRange, label: "Under $5k", desc: "Prototype Sprint" },
                  { id: "5k-25k" as BudgetRange, label: "$5k-25k", desc: "Launch Sprint" },
                  { id: "25k-plus" as BudgetRange, label: "$25k+", desc: "Enterprise Sprint" },
                ].map((option) => (
                  <button
                    key={option.id}
                    className={`qual-option ${qualification.budget === option.id ? 'selected' : ''}`}
                    onClick={() => setQualification(prev => ({ ...prev, budget: option.id }))}
                  >
                    <span className="option-label">{option.label}</span>
                    <span className="option-desc">{option.desc}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="qual-section">
              <h3>Timeline</h3>
              <div className="option-group">
                {[
                  { id: "this-week" as Timeline, label: "This week", desc: "Priority track" },
                  { id: "next-2-weeks" as Timeline, label: "Next 2 weeks", desc: "Standard track" },
                  { id: "flexible" as Timeline, label: "Flexible", desc: "Value track" },
                ].map((option) => (
                  <button
                    key={option.id}
                    className={`qual-option ${qualification.timeline === option.id ? 'selected' : ''}`}
                    onClick={() => setQualification(prev => ({ ...prev, timeline: option.id }))}
                  >
                    <span className="option-label">{option.label}</span>
                    <span className="option-desc">{option.desc}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>

          {qualification.budget && qualification.timeline && (
            <div className="route-button">
              <button
                className="cta-button primary large"
                onClick={() => handleQualification(qualification.budget!, qualification.timeline!)}
              >
                Get My Sprint Option
              </button>
            </div>
          )}
        </div>
      ) : (
        <div className="routing-display">
          {renderRouting()}
          <button className="back-button" onClick={resetQualification}>
            ← Choose Different Options
          </button>
        </div>
      )}

      <style jsx>{`
        .book-sprint-cta {
          background: #0a0a0a;
          border: 1px solid #2a2a2a;
          border-radius: 12px;
          padding: 32px;
          margin: 40px 0;
        }

        .header {
          text-align: center;
          margin-bottom: 32px;
        }

        .value-prop h2 {
          color: #ffffff;
          font-size: 28px;
          font-weight: 600;
          margin: 0 0 8px 0;
          line-height: 1.2;
        }

        .value-prop p {
          color: #888888;
          font-size: 16px;
          margin: 0 0 16px 0;
        }

        .social-proof {
          background: #1a1a1a;
          border: 1px solid #2a2a2a;
          border-radius: 8px;
          padding: 8px 16px;
          display: inline-block;
        }

        .social-proof span {
          color: #00ff88;
          font-size: 14px;
          font-weight: 500;
        }

        .qualification-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 32px;
          margin-bottom: 32px;
        }

        .qual-section h3 {
          color: #ffffff;
          font-size: 18px;
          font-weight: 600;
          margin: 0 0 16px 0;
        }

        .option-group {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .qual-option {
          background: #1a1a1a;
          border: 1px solid #2a2a2a;
          border-radius: 8px;
          padding: 16px;
          text-align: left;
          cursor: pointer;
          transition: all 0.2s ease;
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .qual-option:hover {
          border-color: #00ff88;
          background: #111111;
        }

        .qual-option.selected {
          border-color: #00ff88;
          background: #001a10;
        }

        .option-label {
          color: #ffffff;
          font-weight: 500;
          font-size: 16px;
        }

        .option-desc {
          color: #888888;
          font-size: 14px;
        }

        .route-button {
          text-align: center;
        }

        .cta-button {
          background: #00ff88;
          color: #000000;
          border: none;
          border-radius: 8px;
          padding: 12px 24px;
          font-weight: 600;
          font-size: 16px;
          cursor: pointer;
          text-decoration: none;
          display: inline-block;
          transition: all 0.2s ease;
        }

        .cta-button.large {
          padding: 16px 32px;
          font-size: 18px;
        }

        .cta-button:hover {
          background: #00dd76;
          transform: translateY(-1px);
        }

        .routing-result {
          text-align: center;
          padding: 24px;
          border-radius: 12px;
          margin-bottom: 24px;
        }

        .routing-result.prototype {
          background: linear-gradient(135deg, #001a10, #002815);
          border: 1px solid #00ff88;
        }

        .routing-result.consultation {
          background: linear-gradient(135deg, #1a1a00, #282800);
          border: 1px solid #ffdd00;
        }

        .routing-result.enterprise {
          background: linear-gradient(135deg, #001a1a, #002828);
          border: 1px solid #00ddff;
        }

        .routing-result.decline {
          background: linear-gradient(135deg, #1a1a1a, #2a2a2a);
          border: 1px solid #666666;
        }

        .result-content .icon {
          font-size: 48px;
          margin-bottom: 16px;
        }

        .result-content h3 {
          color: #ffffff;
          font-size: 24px;
          font-weight: 600;
          margin: 0 0 12px 0;
        }

        .result-content p {
          color: #cccccc;
          font-size: 16px;
          margin: 0 0 24px 0;
          line-height: 1.5;
        }

        .result-content .secondary {
          color: #888888;
          font-size: 14px;
          margin: 12px 0 0 0;
        }

        .alternatives {
          display: flex;
          flex-direction: column;
          gap: 8px;
          margin: 16px 0;
        }

        .alternative-link {
          color: #00ff88;
          text-decoration: none;
          font-weight: 500;
          padding: 8px 16px;
          border: 1px solid #2a2a2a;
          border-radius: 6px;
          display: inline-block;
          transition: all 0.2s ease;
        }

        .alternative-link:hover {
          border-color: #00ff88;
          background: #001a10;
        }

        .back-button {
          background: transparent;
          border: 1px solid #2a2a2a;
          color: #888888;
          border-radius: 8px;
          padding: 8px 16px;
          cursor: pointer;
          font-size: 14px;
          transition: all 0.2s ease;
        }

        .back-button:hover {
          border-color: #00ff88;
          color: #00ff88;
        }

        @media (max-width: 768px) {
          .book-sprint-cta {
            padding: 24px 16px;
            margin: 24px -16px;
            border-radius: 0;
            border-left: none;
            border-right: none;
          }

          .qualification-grid {
            grid-template-columns: 1fr;
            gap: 24px;
          }

          .value-prop h2 {
            font-size: 24px;
          }

          .alternatives {
            flex-direction: column;
            align-items: center;
          }
        }
      `}</style>
    </section>
  );
}
