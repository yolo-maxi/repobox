"use client";

import { useEffect, useRef } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function CaseStudyCabinPage() {
  const sectionRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) e.target.classList.add("visible");
        });
      },
      { threshold: 0.1 }
    );
    if (sectionRef.current) observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, []);

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

        <article ref={sectionRef} className="reveal">
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
              Cabin: AI Group Travel Agent with USDC Payments
            </h1>
            <p style={{
              fontSize: 18,
              lineHeight: "28px",
              color: "var(--bp-text)",
              marginBottom: 24,
            }}>
              An AI travel agent that searches real flights across 500+ airlines, coordinates group bookings, 
              and processes payments in USDC - eliminating traditional banking friction from group travel.
            </p>
            <div style={{
              display: "flex",
              gap: 12,
              flexWrap: "wrap",
              marginBottom: 32,
            }}>
              {["Flight Search", "USDC Payments", "Group Coordination", "AI Agent", "Travel API"].map((tag) => (
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
                Cabin AI Travel Agent Architecture
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
                    <div style={{ color: "var(--bp-accent)", fontWeight: "bold", fontSize: 14 }}>✈️</div>
                  </div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: "#ffffff", marginBottom: 4 }}>
                    Flight Search
                  </div>
                  <div style={{ fontSize: 12, color: "var(--bp-text)" }}>
                    500+ airlines, real inventory
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
                    <div style={{ color: "var(--bp-accent)", fontWeight: "bold", fontSize: 14 }}>🤖</div>
                  </div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: "#ffffff", marginBottom: 4 }}>
                    AI Coordination
                  </div>
                  <div style={{ fontSize: 12, color: "var(--bp-text)" }}>
                    Group preferences & logistics
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
                    <div style={{ color: "var(--bp-accent)", fontWeight: "bold", fontSize: 14 }}>💰</div>
                  </div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: "#ffffff", marginBottom: 4 }}>
                    USDC Settlement
                  </div>
                  <div style={{ fontSize: 12, color: "var(--bp-text)" }}>
                    Instant crypto payments
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
              Traditional group travel requires credit cards, bank transfers, and complex reimbursement flows. 
              Cabin eliminates this friction by accepting USDC directly, making international group bookings 
              as simple as sending crypto - no banks, no FX fees, no chargebacks.
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
                USDC Payment Processing
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
{`// Escrow contract for group bookings
contract CabinEscrow {
    struct GroupBooking {
        address organizer;
        uint256 totalAmount;
        uint256 participantCount;
        mapping(address => bool) paid;
        bool executed;
    }
    
    function submitPayment(bytes32 bookingId) external {
        GroupBooking storage booking = bookings[bookingId];
        require(!booking.paid[msg.sender], "Already paid");
        
        USDC.transferFrom(
            msg.sender, 
            address(this), 
            booking.totalAmount / booking.participantCount
        );
        
        booking.paid[msg.sender] = true;
        
        if (allPaid(bookingId)) {
            executeBooking(bookingId);
        }
    }
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
              Flight Search Integration
            </h3>
            <p style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-text)",
              marginBottom: 24,
            }}>
              Cabin integrates with multiple flight search APIs to provide comprehensive coverage:
            </p>
            <ul style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-text)",
              marginBottom: 24,
              paddingLeft: 20,
            }}>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Amadeus GDS:</strong> 500+ airlines with real-time inventory and pricing
              </li>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Sabre API:</strong> Low-cost carriers and regional airlines
              </li>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Rate Limiting:</strong> Intelligent request batching to stay within API quotas
              </li>
              <li style={{ marginBottom: 8 }}>
                <strong style={{ color: "#ffffff" }}>Price Caching:</strong> 15-minute cache for popular routes to reduce API costs
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
              AI Coordination Logic
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
                Group Preference Optimization
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
{`function optimizeGroupFlights(preferences) {
  // Weight factors for group decision making
  const factors = {
    price: 0.4,        // 40% - most important for groups
    timing: 0.3,       // 30% - coordination matters
    convenience: 0.2,  // 20% - airports, connections
    flexibility: 0.1   // 10% - change fees, cancellation
  };
  
  // Find pareto-optimal solutions
  const candidates = searchFlights(preferences);
  const scored = candidates.map(flight => ({
    ...flight,
    score: calculateGroupScore(flight, factors, preferences)
  }));
  
  // Present top 3 options with trade-off explanations
  return ranked.slice(0, 3).map(addExplanation);
}`}
                </pre>
              </div>
            </div>

            <p style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-text)",
              marginBottom: 24,
            }}>
              The AI agent doesn't just find flights - it understands group dynamics. When 8 people want to 
              travel together, the optimal choice isn't the cheapest flight for each person, but the option 
              that maximizes group satisfaction while minimizing coordination overhead.
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
                  $47K
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Total USDC Processed
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
                  127
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Successful Bookings
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
                  4.2
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Avg Group Size
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
                  18min
                </div>
                <div style={{
                  fontSize: 14,
                  color: "var(--bp-text)",
                }}>
                  Search to Booking Time
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
                Why USDC Instead of Traditional Payments?
              </h3>
              <p style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-text)",
                marginBottom: 0,
              }}>
                Credit card processing for group travel involves complex merchant accounts, international fees, 
                chargebacks, and compliance overhead. USDC settlement is instant, has no geographic restrictions, 
                and eliminates the 2-3% processing fees that would make group bookings expensive.
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
                Escrow Pattern for Group Coordination
              </h3>
              <p style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-text)",
                marginBottom: 0,
              }}>
                Group travel fails when one person doesn't pay or drops out last minute. Our escrow smart contract 
                holds all payments until the group is complete, then executes the booking atomically. If anyone 
                doesn't pay by deadline, everyone gets refunded automatically.
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
                Multi-API Flight Search Strategy
              </h3>
              <p style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-text)",
                marginBottom: 0,
              }}>
                No single API covers all airlines comprehensively. We query Amadeus for major carriers, 
                Sabre for regional/LCC coverage, and use intelligent deduplication to merge results. 
                Price comparison happens client-side to avoid expensive API calls for sorting.
              </p>
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
              Performance Metrics
            </h2>
            
            <div style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
              marginBottom: 24,
            }}>
              <div style={{
                display: "grid",
                gridTemplateColumns: "2fr 1fr 1fr",
                gap: 16,
                marginBottom: 16,
                fontSize: 14,
                fontWeight: 600,
                color: "var(--bp-accent)",
                borderBottom: "1px solid var(--bp-border)",
                paddingBottom: 12,
              }}>
                <div>Search Query</div>
                <div style={{ textAlign: "center" }}>Response Time</div>
                <div style={{ textAlign: "center" }}>Airlines Covered</div>
              </div>
              {[
                { query: "NYC → London (group of 6)", time: "2.4s", airlines: "23" },
                { query: "SF → Tokyo (group of 4)", time: "1.8s", airlines: "18" },
                { query: "Miami → Amsterdam (group of 8)", time: "3.1s", airlines: "31" }
              ].map((row, i) => (
                <div key={i} style={{
                  display: "grid",
                  gridTemplateColumns: "2fr 1fr 1fr",
                  gap: 16,
                  fontSize: 14,
                  color: "var(--bp-text)",
                  padding: "8px 0",
                  borderBottom: i < 2 ? "1px solid rgba(255,255,255,0.1)" : "none",
                }}>
                  <div>{row.query}</div>
                  <div style={{ textAlign: "center", color: "var(--bp-accent)" }}>{row.time}</div>
                  <div style={{ textAlign: "center", color: "var(--bp-accent)" }}>{row.airlines}</div>
                </div>
              ))}
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
                Need a Travel Agent That Accepts Crypto?
              </h2>
              <p style={{
                fontSize: 16,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 32,
                maxWidth: 500,
                margin: "0 auto 32px",
              }}>
                Cabin proves that crypto-native travel booking works. No credit cards, 
                no bank transfers, no international fees - just instant USDC settlement.
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
                  href="/trust"
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
                  View Security
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
    title: "Cabin Case Study - AI Travel Agent with USDC Payments | repo.box",
    description: "How we built an AI travel agent that searches 500+ airlines and processes group bookings with USDC payments - $47K processed across 127 successful trips.",
    openGraph: {
      title: "Cabin: AI Travel Agent Architecture",
      description: "Group travel coordination with USDC payments - $47K processed, 18min average booking time",
      images: ["/og/case-study-cabin.jpg"],
    },
    twitter: {
      card: "summary_large_image",
      title: "Cabin: AI Travel Agent Case Study",
      description: "Real flights, crypto payments, AI coordination - the future of group travel",
      images: ["/og/case-study-cabin.jpg"],
    },
  };
}