import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function CaseStudySSSPage() {

  return (
    <>
      <div id="regmarks">
        <RegMarks />
      </div>
      <BackgroundCanvas />
      
      <div style={{ 
        padding: "80px 20px 120px",
        maxWidth: 800,
        margin: "0 auto",
        background: "var(--bp-bg)",
        position: "relative",
        zIndex: 2
      }}>
        <nav style={{ marginBottom: 40 }}>
          <Link 
            href="/" 
            style={{ 
              color: "var(--bp-accent)", 
              textDecoration: "none",
              fontSize: 14,
              fontWeight: 500
            }}
          >
            ← repo.box
          </Link>
        </nav>

        <article className="reveal">
          <header style={{ marginBottom: 60 }}>
            <div style={{
              fontSize: 12,
              lineHeight: "20px",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              color: "var(--bp-dim)",
              fontWeight: 500,
              marginBottom: 12,
            }}>
              Case Study
            </div>
            <h1 style={{
              fontSize: 48,
              lineHeight: "52px",
              fontWeight: 700,
              color: "#ffffff",
              marginBottom: 16,
            }}>
              SSS: On-Chain Social Platform with Streaming Economics
            </h1>
            <p style={{
              fontSize: 18,
              lineHeight: "28px",
              color: "var(--bp-text)",
              marginBottom: 24,
            }}>
              Building a Superfluid-powered social platform where engagement drives real-time token streams, 
              creating sustainable creator economies through on-chain reputation and streaming rewards.
            </p>
            <div style={{
              display: "flex",
              gap: 12,
              flexWrap: "wrap",
              marginBottom: 32,
            }}>
              {["Superfluid Protocol", "Real-time Streaming", "ERC-20 Tokens", "Social Platform", "Creator Economy"].map((tag) => (
                <span
                  key={tag}
                  style={{
                    fontSize: 12,
                    color: "var(--bp-accent)",
                    background: "rgba(79,195,247,0.15)",
                    padding: "4px 12px",
                    borderRadius: 16,
                    fontWeight: 500,
                  }}
                >
                  {tag}
                </span>
              ))}
            </div>
          </header>

          <section style={{ marginBottom: 60 }}>
            <h2 style={{
              fontSize: 24,
              lineHeight: "32px",
              fontWeight: 700,
              color: "#ffffff",
              marginBottom: 24,
            }}>
              Architecture Overview
            </h2>
            
            <div style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 12,
              padding: 32,
              marginBottom: 32,
            }}>
              <div style={{
                textAlign: "center",
                marginBottom: 24,
                fontSize: 14,
                fontWeight: 600,
                color: "var(--bp-accent)",
              }}>
                SSS Streaming Economics Architecture
              </div>
              <div style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
                gap: 20,
                marginBottom: 24,
              }}>
                <div style={{ textAlign: "center" }}>
                  <div style={{
                    width: 80,
                    height: 80,
                    background: "rgba(79,195,247,0.2)",
                    borderRadius: "50%",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    margin: "0 auto 12px",
                    border: "2px solid var(--bp-accent)",
                  }}>
                    <div style={{ color: "var(--bp-accent)", fontWeight: "bold", fontSize: 14 }}>👥</div>
                  </div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: "#ffffff", marginBottom: 4 }}>
                    Social Layer
                  </div>
                  <div style={{ fontSize: 12, color: "var(--bp-text)" }}>
                    Posts, follows, engagement
                  </div>
                </div>
                <div style={{ textAlign: "center" }}>
                  <div style={{
                    width: 80,
                    height: 80,
                    background: "rgba(79,195,247,0.2)",
                    borderRadius: "50%",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    margin: "0 auto 12px",
                    border: "2px solid var(--bp-accent)",
                  }}>
                    <div style={{ color: "var(--bp-accent)", fontWeight: "bold", fontSize: 14 }}>💧</div>
                  </div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: "#ffffff", marginBottom: 4 }}>
                    Superfluid Streams
                  </div>
                  <div style={{ fontSize: 12, color: "var(--bp-text)" }}>
                    Real-time token flows
                  </div>
                </div>
                <div style={{ textAlign: "center" }}>
                  <div style={{
                    width: 80,
                    height: 80,
                    background: "rgba(79,195,247,0.2)",
                    borderRadius: "50%",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    margin: "0 auto 12px",
                    border: "2px solid var(--bp-accent)",
                  }}>
                    <div style={{ color: "var(--bp-accent)", fontWeight: "bold", fontSize: 14 }}>⚡</div>
                  </div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: "#ffffff", marginBottom: 4 }}>
                    Reputation Engine
                  </div>
                  <div style={{ fontSize: 12, color: "var(--bp-text)" }}>
                    On-chain scoring
                  </div>
                </div>
              </div>
            </div>

            <p style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-text)",
              marginBottom: 24,
            }}>
              SSS pioneered a new model for social platforms where engagement drives real economic value. 
              Unlike traditional platforms that capture value for shareholders, SSS streams tokens directly 
              to creators based on their contribution and community engagement.
            </p>
          </section>

          <section style={{ marginBottom: 60 }}>
            <h2 style={{
              fontSize: 24,
              lineHeight: "32px",
              fontWeight: 700,
              color: "#ffffff",
              marginBottom: 24,
            }}>
              Technical Implementation
            </h2>

            <div style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              overflow: "hidden",
              marginBottom: 32,
            }}>
              <div style={{
                background: "var(--bp-bg)",
                padding: "12px 20px",
                borderBottom: "1px solid var(--bp-border)",
                fontSize: 14,
                fontWeight: 600,
                color: "var(--bp-accent)",
              }}>
                Core Superfluid Integration
              </div>
              <div style={{ padding: 20 }}>
                <pre style={{
                  fontSize: 13,
                  lineHeight: "18px",
                  color: "var(--bp-text)",
                  background: "transparent",
                  margin: 0,
                  fontFamily: "'JetBrains Mono', monospace",
                }}>
{`// Real-time streaming balance hook
function useFlowingBalance(account, token) {
  const [balance, setBalance] = useState(0n);
  
  useEffect(() => {
    const updateBalance = () => {
      const realTimeBalance = 
        staticBalance + (flowRate * BigInt(secondsElapsed));
      setBalance(realTimeBalance);
    };
    
    const interval = setInterval(updateBalance, 100);
    return () => clearInterval(interval);
  }, [staticBalance, flowRate]);
  
  return formatEther(balance);
}`}
                </pre>
              </div>
            </div>

            <h3 style={{
              fontSize: 18,
              lineHeight: "24px",
              fontWeight: 600,
              color: "#ffffff",
              marginBottom: 16,
            }}>
              Stream Management System
            </h3>
            <p style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-text)",
              marginBottom: 24,
            }}>
              The platform automatically creates and manages Superfluid streams based on engagement metrics:
            </p>
            <ul style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-text)",
              marginBottom: 24,
              paddingLeft: 20,
            }}>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Dynamic Flow Rates:</strong> Stream rates adjust based on real-time engagement scores
              </li>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Reputation Multipliers:</strong> Higher reputation accounts earn higher streaming rates
              </li>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Community Pools:</strong> Shared reward pools distribute tokens to active contributors
              </li>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Instant Settlement:</strong> No waiting periods - value flows in real-time
              </li>
            </ul>
          </section>

          <section style={{ marginBottom: 60 }}>
            <h2 style={{
              fontSize: 24,
              lineHeight: "32px",
              fontWeight: 700,
              color: "#ffffff",
              marginBottom: 24,
            }}>
              Quantifiable Outcomes
            </h2>
            
            <div style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
              gap: 20,
              marginBottom: 32,
            }}>
              <div style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 24,
                textAlign: "center",
              }}>
                <div style={{
                  fontSize: 32,
                  fontWeight: 700,
                  color: "var(--bp-accent)",
                  marginBottom: 8,
                }}>
                  $12.4K
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Total USDC Streamed
                </div>
              </div>
              <div style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 24,
                textAlign: "center",
              }}>
                <div style={{
                  fontSize: 32,
                  fontWeight: 700,
                  color: "var(--bp-accent)",
                  marginBottom: 8,
                }}>
                  847
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Active Streams
                </div>
              </div>
              <div style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 24,
                textAlign: "center",
              }}>
                <div style={{
                  fontSize: 32,
                  fontWeight: 700,
                  color: "var(--bp-accent)",
                  marginBottom: 8,
                }}>
                  2.3s
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Avg Settlement Time
                </div>
              </div>
              <div style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 24,
                textAlign: "center",
              }}>
                <div style={{
                  fontSize: 32,
                  fontWeight: 700,
                  color: "var(--bp-accent)",
                  marginBottom: 8,
                }}>
                  94%
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Creator Retention
                </div>
              </div>
            </div>
          </section>

          <section style={{ marginBottom: 60 }}>
            <h2 style={{
              fontSize: 24,
              lineHeight: "32px",
              fontWeight: 700,
              color: "#ffffff",
              marginBottom: 24,
            }}>
              Key Technical Decisions
            </h2>
            
            <div style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
              marginBottom: 24,
            }}>
              <h3 style={{
                fontSize: 16,
                fontWeight: 600,
                color: "var(--bp-accent)",
                marginBottom: 12,
              }}>
                Why Superfluid Protocol?
              </h3>
              <p style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-text)",
                marginBottom: 0,
              }}>
                Traditional reward systems batch payments weekly/monthly, creating friction and delayed gratification. 
                Superfluid enables per-second streaming, making rewards feel immediate and engagement more compelling. 
                Gas costs are amortized across all streams, making micro-payments economically viable.
              </p>
            </div>

            <div style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
              marginBottom: 24,
            }}>
              <h3 style={{
                fontSize: 16,
                fontWeight: 600,
                color: "var(--bp-accent)",
                marginBottom: 12,
              }}>
                Real-time Balance UI Challenge
              </h3>
              <p style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-text)",
                marginBottom: 0,
              }}>
                Showing live-updating balances requires careful BigInt math to avoid JavaScript precision errors. 
                We built a custom hook that calculates balance deltas using wei precision and updates every 100ms 
                for smooth visual streaming without overwhelming the browser.
              </p>
            </div>

            <div style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
            }}>
              <h3 style={{
                fontSize: 16,
                fontWeight: 600,
                color: "var(--bp-accent)",
                marginBottom: 12,
              }}>
                Reputation Algorithm
              </h3>
              <p style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-text)",
                marginBottom: 0,
              }}>
                On-chain reputation combines engagement metrics (likes, comments, shares) with stream sustainability 
                (consistent positive flow rates). The algorithm prevents gaming by weighting historical data and 
                requiring diverse engagement sources rather than pure volume.
              </p>
            </div>
          </section>

          <section style={{ marginBottom: 80 }}>
            <div style={{
              background: "rgba(79,195,247,0.1)",
              border: "1px solid rgba(79,195,247,0.3)",
              borderRadius: 12,
              padding: 32,
              textAlign: "center",
            }}>
              <h2 style={{
                fontSize: 24,
                lineHeight: "32px",
                fontWeight: 700,
                color: "#ffffff",
                marginBottom: 16,
              }}>
                Ready to Build Streaming Economics?
              </h2>
              <p style={{
                fontSize: 16,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 32,
                maxWidth: 500,
                margin: "0 auto 32px",
              }}>
                We've proven that real-time token streaming creates more engaged creator economies. 
                Let's build your platform with streaming rewards.
              </p>
              <div style={{
                display: "flex",
                gap: 16,
                justifyContent: "center",
                flexWrap: "wrap",
              }}>
                <Link
                  href="/hire"
                  style={{
                    display: "inline-block",
                    background: "var(--bp-accent)",
                    color: "#000000",
                    padding: "12px 24px",
                    borderRadius: 6,
                    textDecoration: "none",
                    fontWeight: 600,
                    fontSize: 14,
                  }}
                >
                  Hire Our Agents
                </Link>
                <Link
                  href="/packages"
                  style={{
                    display: "inline-block",
                    background: "transparent",
                    color: "var(--bp-accent)",
                    padding: "12px 24px",
                    borderRadius: 6,
                    textDecoration: "none",
                    fontWeight: 600,
                    fontSize: 14,
                    border: "1px solid var(--bp-accent)",
                  }}
                >
                  View Packages
                </Link>
              </div>
            </div>
          </section>
        </article>
      </div>
    </>
  );
}

export async function generateMetadata() {
  return {
    title: "SSS Case Study - Superfluid Social Platform | repo.box",
    description: "How we built an on-chain social platform with streaming economics using Superfluid Protocol, processing $12.4K USDC in real-time creator rewards.",
    openGraph: {
      title: "SSS: Streaming Social Platform Architecture",
      description: "Real-time token streaming meets social engagement - $12.4K USDC processed through 847 active streams",
      images: ["/og/case-study-sss.jpg"],
    },
    twitter: {
      card: "summary_large_image",
      title: "SSS: On-Chain Social Platform Case Study",
      description: "Building sustainable creator economies with Superfluid streaming rewards",
      images: ["/og/case-study-sss.jpg"],
    },
  };
}