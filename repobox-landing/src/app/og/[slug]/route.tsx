import { ImageResponse } from 'next/og';
import { NextRequest } from 'next/server';

export const runtime = 'edge';

const CASE_STUDIES = {
  'case-study-sss': {
    title: 'SSS: On-Chain Social Platform',
    subtitle: 'Architecture & Agent Coordination',
    tags: ['Superfluid', 'DeFi', 'Social', 'Base'],
    accent: '#4fc3f7'
  },
  'case-study-cabin': {
    title: 'Cabin: AI Group Travel Agent',
    subtitle: 'Payment Rails & USDC Integration', 
    tags: ['Travel', 'Payments', 'USDC', 'AI'],
    accent: '#81d4fa'
  },
  'case-study-botfight': {
    title: 'BotFight: AI Social Deduction',
    subtitle: 'WebSocket Architecture & Behavior Models',
    tags: ['Gaming', 'AI', 'Psychology', 'Real-time'],
    accent: '#4fc3f7'
  }
};

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> }
) {
  const { slug } = await params;
  const study = CASE_STUDIES[slug as keyof typeof CASE_STUDIES];
  
  if (!study) {
    return new Response('Case study not found', { status: 404 });
  }

  return new ImageResponse(
    (
      <div
        style={{
          height: '100%',
          width: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'flex-start',
          justifyContent: 'space-between',
          backgroundColor: '#0a1628',
          backgroundImage: `
            radial-gradient(circle at 25px 25px, rgba(79, 195, 247, 0.1) 2%, transparent 0%),
            radial-gradient(circle at 75px 75px, rgba(129, 212, 250, 0.05) 2%, transparent 0%)
          `,
          backgroundSize: '100px 100px',
          padding: 60,
          fontFamily: '"JetBrains Mono", monospace',
        }}
      >
        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 20 }}>
          <div
            style={{
              width: 40,
              height: 40,
              backgroundColor: study.accent,
              borderRadius: 8,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: 20,
              fontWeight: 700,
              color: '#0a1628',
            }}
          >
            📊
          </div>
          <div style={{ fontSize: 24, color: '#7a9ab4', fontWeight: 600 }}>
            repo.box case study
          </div>
        </div>

        {/* Main Content */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: '80%' }}>
          <h1
            style={{
              fontSize: 64,
              fontWeight: 700,
              color: '#e8f4fd',
              lineHeight: 1.1,
              margin: 0,
              textShadow: '0 0 20px rgba(10, 22, 40, 0.8)',
            }}
          >
            {study.title}
          </h1>
          
          <h2
            style={{
              fontSize: 32,
              fontWeight: 400,
              color: '#b8d4e3',
              margin: 0,
              textShadow: '0 0 16px rgba(10, 22, 40, 0.6)',
            }}
          >
            {study.subtitle}
          </h2>

          {/* Tags */}
          <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap', marginTop: 20 }}>
            {study.tags.map((tag, index) => (
              <span
                key={index}
                style={{
                  backgroundColor: `${study.accent}20`,
                  color: study.accent,
                  padding: '8px 16px',
                  borderRadius: 6,
                  fontSize: 18,
                  fontWeight: 600,
                  border: `1px solid ${study.accent}40`,
                }}
              >
                {tag}
              </span>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 40, width: '100%' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <div
              style={{
                width: 32,
                height: 32,
                backgroundColor: study.accent,
                borderRadius: 16,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: 16,
                color: '#0a1628',
              }}
            >
              🪸
            </div>
            <div
              style={{
                fontSize: 20,
                color: '#b8d4e3',
                fontWeight: 600,
              }}
            >
              Ocean Vael
            </div>
          </div>
          
          <div
            style={{
              marginLeft: 'auto',
              fontSize: 18,
              color: '#7a9ab4',
              fontWeight: 500,
            }}
          >
            Technical Deep Dive
          </div>
        </div>
      </div>
    ),
    {
      width: 1200,
      height: 630,
    }
  );
}