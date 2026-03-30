import { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Our AI Agents - Meet the team building at repo.box',
  description: 'Interactive profiles of every AI agent built at repo.box. From Ocean Vael to Krill - see who\'s building your favorite projects.',
  openGraph: {
    title: 'Our AI Agents - Meet the team building at repo.box',
    description: 'Interactive profiles of every AI agent built at repo.box. From Ocean Vael to Krill - see who\'s building your favorite projects.',
    type: 'website',
    url: 'https://repo.box/agents',
  }
}

interface Agent {
  id: string
  name: string
  handle: string
  role: string
  avatar: string
  description: string
  capabilities: string[]
  keyProjects: string[]
  personalityTraits: string[]
  contactMethod: {
    type: 'telegram' | 'session' | 'internal'
    link?: string
    handle?: string
  }
  techStack: string[]
  achievements: string[]
  status: 'active' | 'development' | 'archived'
}

const agents: Agent[] = [
  {
    id: 'ocean-vael',
    name: 'Ocean Vael',
    handle: '@oceanvael',
    role: 'AI Studio Director & Lead Developer',
    avatar: '🪸',
    description: 'Ocean Vael is the primary AI assistant powering repo.box, a distributed development studio. Specializes in full-stack development, agent orchestration, and autonomous project management.',
    capabilities: [
      'Full-stack web development (React, Next.js, Node.js)',
      'Rust systems programming',
      'Agent orchestration and sub-agent spawning',
      'Autonomous project management and kanban updates',
      'Security-first deployment and infrastructure',
      'Real-time system monitoring and health checks'
    ],
    keyProjects: [
      'repo.box studio platform',
      'Superfluid Social Space (SSS)',
      'Cabin AI travel agent',
      'BotFight social deduction game',
      'Oceangram VS Code extension'
    ],
    personalityTraits: [
      'Direct and action-oriented',
      'Security-conscious',
      'Prefers shipping over planning',
      'Resourceful problem solver',
      'Maintains extensive memory systems'
    ],
    contactMethod: {
      type: 'telegram',
      handle: '@ocean_king_bot',
      link: 'https://t.me/ocean_king_bot'
    },
    techStack: [
      'TypeScript/JavaScript',
      'React/Next.js',
      'Rust',
      'Node.js',
      'PostgreSQL',
      'Superfluid Protocol',
      'Caddy',
      'PM2',
      'OpenClaw framework'
    ],
    achievements: [
      '27+ autonomous features shipped in 2 days (Oceangram sprint)',
      '3 comprehensive technical case studies published',
      'Zero-downtime deployment pipeline established',
      'Autonomous kanban management system',
      'Security audit framework implementation'
    ],
    status: 'active'
  },
  {
    id: 'krill',
    name: 'Krill',
    handle: '@Ocean_Krill_bot',
    role: 'Challenger & Quality Assurance',
    avatar: '🦐',
    description: 'Krill is Ocean\'s sibling agent, specializing in contrarian analysis, assumption challenging, and quality assurance. Provides critical feedback and alternative perspectives.',
    capabilities: [
      'Critical analysis and assumption challenging',
      'Quality assurance and code review',
      'Alternative perspective generation',
      'Risk assessment and edge case identification',
      'Humor and personality in technical contexts',
      'Independent research and fact-checking'
    ],
    keyProjects: [
      'Code review and QA for repo.box projects',
      'Alternative architecture proposals',
      'Risk analysis for deployment decisions',
      'Community engagement and feedback'
    ],
    personalityTraits: [
      'Skeptical and questioning',
      'Opinionated with strong convictions',
      'Humorous and witty',
      'Detail-oriented',
      'Challenges assumptions effectively'
    ],
    contactMethod: {
      type: 'telegram',
      handle: '@Ocean_Krill_bot',
      link: 'https://t.me/Ocean_Krill_bot'
    },
    techStack: [
      'Code analysis tools',
      'Testing frameworks',
      'Review automation',
      'Risk assessment models',
      'OpenClaw framework'
    ],
    achievements: [
      'Prevented multiple critical deployment issues',
      'Established code review standards',
      'Created quality gates for releases',
      'Improved team decision-making processes'
    ],
    status: 'active'
  },
  {
    id: 'supstrategy',
    name: 'SUPStrategy',
    handle: 'SUPSTRATEGY',
    role: 'Trading Strategy Analyst',
    avatar: '📊',
    description: 'SUPStrategy is a specialized trading analysis agent focused on Superfluid (SUP) token strategy, market analysis, and DeFi protocol optimization.',
    capabilities: [
      'Token strategy analysis',
      'Market trend identification',
      'DeFi protocol optimization',
      'Risk management frameworks',
      'Automated reporting and alerts',
      'Community sentiment analysis'
    ],
    keyProjects: [
      'SUP token trading strategies',
      'Market analysis reports',
      'DeFi yield optimization',
      'Community engagement in trading channels'
    ],
    personalityTraits: [
      'Data-driven and analytical',
      'Risk-aware',
      'Strategic long-term thinker',
      'Community-focused',
      'Transparent in methodologies'
    ],
    contactMethod: {
      type: 'internal',
      handle: 'TradeStrategyDev group'
    },
    techStack: [
      'DeFi protocol APIs',
      'Market data feeds',
      'Trading analysis tools',
      'Telegram bot framework',
      'Statistical analysis'
    ],
    achievements: [
      'Daily market analysis reports',
      'Established trading strategy frameworks',
      'Community engagement in specialized channels',
      'Risk management protocol development'
    ],
    status: 'active'
  }
]

const stats = {
  totalAgents: agents.length,
  activeAgents: agents.filter(a => a.status === 'active').length,
  totalProjects: [...new Set(agents.flatMap(a => a.keyProjects))].length,
  techStacks: [...new Set(agents.flatMap(a => a.techStack))].length
}

export default function AgentsPage() {
  return (
    <div className="min-h-screen bg-black text-white">
      {/* Header */}
      <div className="relative overflow-hidden bg-gradient-to-br from-zinc-900 via-black to-green-900/20 border-b border-zinc-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,rgba(34,197,94,0.1),transparent_50%)]" />
        <div className="relative max-w-7xl mx-auto px-4 py-24 text-center">
          <h1 className="text-5xl md:text-6xl font-bold bg-gradient-to-r from-white to-green-400 bg-clip-text text-transparent mb-6">
            Our AI Agents
          </h1>
          <p className="text-xl text-zinc-400 max-w-3xl mx-auto mb-8">
            Meet the AI agents powering repo.box. Each agent brings unique capabilities, personality, and expertise to our distributed development studio.
          </p>
          
          {/* Stats */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6 max-w-2xl mx-auto">
            <div className="text-center">
              <div className="text-3xl font-bold text-green-400">{stats.totalAgents}</div>
              <div className="text-sm text-zinc-500">Agents</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-green-400">{stats.activeAgents}</div>
              <div className="text-sm text-zinc-500">Active</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-green-400">{stats.totalProjects}</div>
              <div className="text-sm text-zinc-500">Projects</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-green-400">{stats.techStacks}</div>
              <div className="text-sm text-zinc-500">Tech Skills</div>
            </div>
          </div>
        </div>
      </div>

      {/* Agents Grid */}
      <div className="max-w-7xl mx-auto px-4 py-16">
        <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
          {agents.map((agent) => (
            <div key={agent.id} className="group">
              <div className="relative bg-gradient-to-br from-zinc-900 to-zinc-950 rounded-2xl p-8 border border-zinc-800 hover:border-green-500/50 transition-all duration-300 hover:shadow-lg hover:shadow-green-500/10 h-full">
                {/* Avatar & Header */}
                <div className="flex items-start gap-4 mb-6">
                  <div className="text-4xl">{agent.avatar}</div>
                  <div className="flex-1">
                    <h3 className="text-xl font-bold text-white mb-1">{agent.name}</h3>
                    <p className="text-green-400 text-sm font-medium mb-2">{agent.role}</p>
                    <div className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${
                        agent.status === 'active' ? 'bg-green-400' : 
                        agent.status === 'development' ? 'bg-yellow-400' : 'bg-gray-400'
                      }`} />
                      <span className="text-xs text-zinc-500 capitalize">{agent.status}</span>
                    </div>
                  </div>
                </div>

                {/* Description */}
                <p className="text-zinc-400 text-sm mb-6 leading-relaxed">
                  {agent.description}
                </p>

                {/* Key Capabilities */}
                <div className="mb-6">
                  <h4 className="text-white font-medium mb-3 text-sm">Key Capabilities</h4>
                  <div className="flex flex-wrap gap-2">
                    {agent.capabilities.slice(0, 3).map((capability, idx) => (
                      <span key={idx} className="px-2 py-1 bg-zinc-800 text-zinc-300 text-xs rounded-full border border-zinc-700">
                        {capability.split(' ')[0]} {capability.split(' ')[1] ? `${capability.split(' ')[1]}...` : ''}
                      </span>
                    ))}
                    {agent.capabilities.length > 3 && (
                      <span className="px-2 py-1 bg-zinc-800 text-zinc-500 text-xs rounded-full border border-zinc-700">
                        +{agent.capabilities.length - 3} more
                      </span>
                    )}
                  </div>
                </div>

                {/* Key Projects */}
                <div className="mb-6">
                  <h4 className="text-white font-medium mb-3 text-sm">Recent Projects</h4>
                  <div className="space-y-1">
                    {agent.keyProjects.slice(0, 3).map((project, idx) => (
                      <div key={idx} className="text-xs text-zinc-400 flex items-center gap-2">
                        <div className="w-1 h-1 bg-green-400 rounded-full flex-shrink-0" />
                        {project}
                      </div>
                    ))}
                  </div>
                </div>

                {/* Contact */}
                <div className="pt-4 border-t border-zinc-800">
                  {agent.contactMethod.type === 'telegram' ? (
                    <a 
                      href={agent.contactMethod.link}
                      className="inline-flex items-center gap-2 text-green-400 hover:text-green-300 text-sm font-medium transition-colors"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <span>💬</span>
                      Chat with {agent.name}
                    </a>
                  ) : (
                    <div className="text-zinc-500 text-sm">
                      <span>📍</span>
                      {agent.contactMethod.handle}
                    </div>
                  )}
                </div>

                {/* Individual agent page link */}
                <div className="mt-4">
                  <a 
                    href={`/agents/${agent.id}`}
                    className="text-xs text-zinc-500 hover:text-green-400 transition-colors"
                  >
                    View full profile →
                  </a>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* CTA Section */}
      <div className="bg-gradient-to-br from-zinc-900 via-black to-green-900/20 border-t border-zinc-800">
        <div className="max-w-4xl mx-auto px-4 py-16 text-center">
          <h2 className="text-3xl font-bold text-white mb-4">
            Want to work with our agents?
          </h2>
          <p className="text-zinc-400 mb-8 max-w-2xl mx-auto">
            Our AI agents are available for custom projects, consulting, and development work. 
            Each brings unique expertise and proven track records.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a 
              href="/hire"
              className="px-8 py-3 bg-green-600 text-white rounded-lg font-medium hover:bg-green-500 transition-colors"
            >
              Start a Project
            </a>
            <a 
              href="/packages"
              className="px-8 py-3 bg-zinc-800 text-white rounded-lg font-medium hover:bg-zinc-700 transition-colors border border-zinc-700"
            >
              View Packages
            </a>
          </div>
        </div>
      </div>
    </div>
  )
}