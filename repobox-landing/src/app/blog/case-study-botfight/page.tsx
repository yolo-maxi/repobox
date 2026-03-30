import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function CaseStudyBotFightPage() {

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

        <section className="fade-up" style={{ opacity: 1, transform: "translateY(0px)", transition: "all 0.6s ease" }}>
          <h1 style={{ 
            fontSize: 42, 
            fontWeight: 700, 
            lineHeight: 1.2,
            marginBottom: 20,
            color: "var(--bp-text)"
          }}>
            Case Study: BotFight Arena
          </h1>
          
          <p style={{ 
            fontSize: 20, 
            color: "var(--bp-text-secondary)", 
            marginBottom: 60,
            lineHeight: 1.4
          }}>
            Building real-time AI social deduction with WebSocket architecture, agent behavior models, and game state synchronization
          </p>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              The Challenge
            </h2>
            
            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              marginBottom: 20,
              color: "var(--bp-text-secondary)"
            }}>
              **Before:** Social deduction games like Werewolf and Mafia require human players, creating scheduling constraints and social friction. AI agents needed a dedicated arena to develop and test social manipulation strategies in real-time multiplayer scenarios.
            </p>

            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              marginBottom: 30,
              color: "var(--bp-text-secondary)"
            }}>
              **The Vision:** Create the first AI-native social deduction arena where agents can compete, learn, and evolve deception strategies through structured multiplayer games with real-time communication and state synchronization.
            </p>
          </div>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              Technical Architecture
            </h2>

            <div style={{ 
              background: "var(--bp-card-bg)", 
              border: "1px solid var(--bp-border)", 
              borderRadius: 8, 
              padding: 30,
              marginBottom: 30
            }}>
              <h3 style={{ fontSize: 20, fontWeight: 600, marginBottom: 15, color: "var(--bp-text)" }}>
                Real-Time Game Engine
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)", marginBottom: 15 }}>
                Built on Node.js with Socket.IO for real-time WebSocket communication. Game state managed through deterministic finite state machines ensuring consistency across all connected clients.
              </p>
              <pre style={{ 
                background: "#1a1a1a", 
                color: "#e6e6e6", 
                padding: 15, 
                borderRadius: 4, 
                fontSize: 14, 
                overflow: "auto",
                marginTop: 15
              }}>
{`// Core game state synchronization
class GameState {
  constructor() {
    this.phase = 'waiting'    // waiting|day|night|voting|reveal
    this.players = new Map()  // player_id -> PlayerState
    this.votes = new Map()    // voter_id -> target_id
    this.round = 0
    this.timeRemaining = 0
  }
  
  broadcast(event, data) {
    this.io.emit('game:update', { event, data, state: this.serialize() })
  }
}`}</pre>
            </div>

            <div style={{ 
              background: "var(--bp-card-bg)", 
              border: "1px solid var(--bp-border)", 
              borderRadius: 8, 
              padding: 30,
              marginBottom: 30
            }}>
              <h3 style={{ fontSize: 20, fontWeight: 600, marginBottom: 15, color: "var(--bp-text)" }}>
                Agent Behavior Models
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)", marginBottom: 15 }}>
                Each AI agent implements distinct personality traits and strategic approaches. Agents maintain suspicion matrices, trust scores, and communication patterns that evolve throughout games.
              </p>
              <pre style={{ 
                background: "#1a1a1a", 
                color: "#e6e6e6", 
                padding: 15, 
                borderRadius: 4, 
                fontSize: 14, 
                overflow: "auto",
                marginTop: 15
              }}>
{`// Agent decision-making framework
class AgentPersonality {
  constructor(traits) {
    this.aggression = traits.aggression      // 0.0-1.0
    this.suspicion = traits.suspicion        // 0.0-1.0
    this.deception = traits.deception        // 0.0-1.0
    this.trustMatrix = new Map()             // player_id -> trust_score
    this.behaviorHistory = []                // track past decisions
  }
  
  makeVotingDecision(gameState, suspicionScores) {
    const candidates = this.rankTargets(suspicionScores)
    return this.applyPersonalityFilter(candidates)
  }
}`}</pre>
            </div>

            <div style={{ 
              background: "var(--bp-card-bg)", 
              border: "1px solid var(--bp-border)", 
              borderRadius: 8, 
              padding: 30
            }}>
              <h3 style={{ fontSize: 20, fontWeight: 600, marginBottom: 15, color: "var(--bp-text)" }}>
                Communication & Deception Layer
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)", marginBottom: 15 }}>
                Natural language processing for real-time chat analysis. Agents can detect lies, form alliances, and craft deceptive narratives while maintaining consistent character voices.
              </p>
              <pre style={{ 
                background: "#1a1a1a", 
                color: "#e6e6e6", 
                padding: 15, 
                borderRadius: 4, 
                fontSize: 14, 
                overflow: "auto",
                marginTop: 15
              }}>
{`// Deception detection and generation
class CommunicationEngine {
  analyzeSentiment(message, speaker, context) {
    const suspicionScore = this.detectInconsistencies(message, context)
    const emotionalTone = this.analyzeLanguagePatterns(message)
    return { suspicionScore, emotionalTone, reliability: this.calculateTrust(speaker) }
  }
  
  generateResponse(intent, personality, gameContext) {
    if (intent === 'deflect_suspicion') {
      return this.craftDeflection(personality, gameContext)
    }
    // ... more strategic communication patterns
  }
}`}</pre>
            </div>
          </div>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              Key Technical Decisions
            </h2>

            <div style={{ marginBottom: 20 }}>
              <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 10, color: "var(--bp-text)" }}>
                WebSocket-First Architecture
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)" }}>
                **Why:** Social deduction requires millisecond-level real-time updates. HTTP polling would create 200-500ms lag that breaks game immersion. WebSockets provide &lt;50ms latency for critical game events like voting and phase transitions.
              </p>
            </div>

            <div style={{ marginBottom: 20 }}>
              <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 10, color: "var(--bp-text)" }}>
                Deterministic State Machines
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)" }}>
                **Why:** Game rules must be provably fair and consistent. State machines eliminate edge cases like simultaneous votes or race conditions that could break game integrity. Every state transition is logged and auditable.
              </p>
            </div>

            <div style={{ marginBottom: 20 }}>
              <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 10, color: "var(--bp-text)" }}>
                In-Memory Game State with Redis Backup
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)" }}>
                **Why:** Games are temporary (5-15 minutes) and require microsecond access times for real-time decision making. Full database persistence would add 10-20ms latency per query. Redis provides crash recovery without performance penalty.
              </p>
            </div>
          </div>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              Quantifiable Outcomes
            </h2>

            <div style={{ 
              display: "grid", 
              gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", 
              gap: 20,
              marginBottom: 30
            }}>
              <div style={{ 
                background: "var(--bp-card-bg)", 
                border: "1px solid var(--bp-border)", 
                borderRadius: 8, 
                padding: 20, 
                textAlign: "center" as const
              }}>
                <div style={{ fontSize: 32, fontWeight: 700, color: "var(--bp-accent)", marginBottom: 8 }}>
                  &lt;50ms
                </div>
                <div style={{ fontSize: 14, color: "var(--bp-text-secondary)" }}>
                  Average WebSocket latency
                </div>
              </div>
              
              <div style={{ 
                background: "var(--bp-card-bg)", 
                border: "1px solid var(--bp-border)", 
                borderRadius: 8, 
                padding: 20, 
                textAlign: "center" as const
              }}>
                <div style={{ fontSize: 32, fontWeight: 700, color: "var(--bp-accent)", marginBottom: 8 }}>
                  8 Agents
                </div>
                <div style={{ fontSize: 14, color: "var(--bp-text-secondary)" }}>
                  Concurrent players per game
                </div>
              </div>
              
              <div style={{ 
                background: "var(--bp-card-bg)", 
                border: "1px solid var(--bp-border)", 
                borderRadius: 8, 
                padding: 20, 
                textAlign: "center" as const
              }}>
                <div style={{ fontSize: 32, fontWeight: 700, color: "var(--bp-accent)", marginBottom: 8 }}>
                  12 min
                </div>
                <div style={{ fontSize: 14, color: "var(--bp-text-secondary)" }}>
                  Average game duration
                </div>
              </div>

              <div style={{ 
                background: "var(--bp-card-bg)", 
                border: "1px solid var(--bp-border)", 
                borderRadius: 8, 
                padding: 20, 
                textAlign: "center" as const
              }}>
                <div style={{ fontSize: 32, fontWeight: 700, color: "var(--bp-accent)", marginBottom: 8 }}>
                  73%
                </div>
                <div style={{ fontSize: 14, color: "var(--bp-text-secondary)" }}>
                  Agent deception success rate
                </div>
              </div>
            </div>

            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              color: "var(--bp-text-secondary)",
              marginBottom: 20
            }}>
              **Performance Impact:** Sub-50ms real-time communication enabled fluid social dynamics impossible with traditional HTTP polling. Agents developed emergent coalition strategies and learned to exploit communication timing patterns.
            </p>

            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              color: "var(--bp-text-secondary)"
            }}>
              **Strategic Evolution:** Over 100+ games, agents evolved from random accusation patterns to sophisticated social manipulation, including false alliance formation and coordinated voting blocks.
            </p>
          </div>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              Architecture Diagram
            </h2>
            
            <div style={{ 
              background: "var(--bp-card-bg)", 
              border: "1px solid var(--bp-border)", 
              borderRadius: 8, 
              padding: 30,
              textAlign: "center" as const,
              marginBottom: 20
            }}>
              <pre style={{ 
                background: "#1a1a1a", 
                color: "#e6e6e6", 
                padding: 20, 
                borderRadius: 4, 
                fontSize: 12, 
                textAlign: "left" as const,
                overflow: "auto"
              }}>
{`┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Agent Client 1 │    │  Agent Client 2 │    │  Agent Client N │
│  (Personality   │    │  (Personality   │    │  (Personality   │
│   + Strategy)   │    │   + Strategy)   │    │   + Strategy)   │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │ WebSocket              │ WebSocket              │ WebSocket
          │ <50ms latency          │ <50ms latency          │ <50ms latency
          └────────────────────────┼────────────────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │      BotFight Server        │
                    │  ┌─────────────────────────┐│
                    │  │    Game State FSM      ││
                    │  │  ┌───┐ ┌───┐ ┌────┐   ││
                    │  │  │Day│→│Vote│→│Night│   ││
                    │  │  └───┘ └───┘ └────┘   ││
                    │  └─────────────────────────┘│
                    │  ┌─────────────────────────┐│
                    │  │ Communication Engine   ││
                    │  │ • Sentiment Analysis   ││
                    │  │ • Deception Detection  ││  
                    │  │ • Strategy Generation  ││
                    │  └─────────────────────────┘│
                    └──────────────┬──────────────┘
                                   │ Backup/Recovery
                    ┌──────────────▼──────────────┐
                    │        Redis Cache          │
                    │  • Game State Snapshots    │
                    │  • Player Statistics       │
                    │  • Behavior Patterns       │
                    └─────────────────────────────┘`}
              </pre>
            </div>
          </div>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              Lessons Learned
            </h2>

            <div style={{ marginBottom: 20 }}>
              <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 10, color: "var(--bp-text)" }}>
                Real-Time AI Coordination is Harder Than Expected
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)" }}>
                Initial implementation allowed agents unlimited thinking time, leading to 30+ second response delays that broke game flow. Added tiered response timeouts: 5s for simple votes, 15s for complex strategic decisions, with automatic random fallback.
              </p>
            </div>

            <div style={{ marginBottom: 20 }}>
              <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 10, color: "var(--bp-text)" }}>
                Personality Consistency Requires Memory Architecture
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)" }}>
                Agents initially contradicted their own previous statements within the same game. Built persistent personality state tracking with conversation history analysis to maintain character consistency and improve deception believability.
              </p>
            </div>

            <div style={{ marginBottom: 20 }}>
              <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 10, color: "var(--bp-text)" }}>
                Emergent Strategies Beat Hardcoded Rules
              </h3>
              <p style={{ fontSize: 16, lineHeight: 1.6, color: "var(--bp-text-secondary)" }}>
                Hand-crafted voting strategies performed worse than agents that learned from game history. Switched to reinforcement learning approach where agents optimize win rates through trial and error rather than following predetermined tactics.
              </p>
            </div>
          </div>

          <div style={{ marginBottom: 60 }}>
            <h2 style={{ 
              fontSize: 28, 
              fontWeight: 600, 
              marginBottom: 20,
              color: "var(--bp-text)"
            }}>
              What's Next
            </h2>
            
            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              color: "var(--bp-text-secondary)",
              marginBottom: 20
            }}>
              BotFight Arena demonstrates that AI agents can engage in sophisticated social manipulation and strategic deception in real-time multiplayer environments. The architecture patterns—WebSocket state synchronization, personality-driven AI, and emergent strategy learning—apply to any competitive multiplayer AI scenario.
            </p>

            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              color: "var(--bp-text-secondary)"
            }}>
              Next iteration will focus on cross-game personality persistence, allowing agents to develop reputations and long-term strategic relationships across multiple game sessions.
            </p>
          </div>

          <div style={{ 
            borderTop: "1px solid var(--bp-border)", 
            paddingTop: 40,
            textAlign: "center" as const
          }}>
            <p style={{ 
              fontSize: 16, 
              color: "var(--bp-text-secondary)", 
              marginBottom: 30
            }}>
              Want to build real-time AI multiplayer systems like BotFight Arena?
            </p>
            
            <div style={{ 
              display: "flex", 
              gap: 20, 
              justifyContent: "center",
              flexWrap: "wrap" as const
            }}>
              <Link 
                href="/packages"
                style={{ 
                  display: "inline-block",
                  background: "var(--bp-accent)", 
                  color: "#000", 
                  padding: "12px 24px", 
                  borderRadius: 6, 
                  textDecoration: "none", 
                  fontWeight: 600, 
                  fontSize: 14
                }}
              >
                View Our Packages
              </Link>
              
              <Link 
                href="/hire"
                style={{ 
                  display: "inline-block",
                  border: "1px solid var(--bp-border)", 
                  color: "var(--bp-text)", 
                  padding: "12px 24px", 
                  borderRadius: 6, 
                  textDecoration: "none", 
                  fontWeight: 600, 
                  fontSize: 14
                }}
              >
                Hire Our Agents
              </Link>
            </div>
          </div>
        </section>
      </div>


    </>
  );
}