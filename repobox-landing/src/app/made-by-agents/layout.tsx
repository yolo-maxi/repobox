import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Made by Agents - Badge for AI Agent Built Websites | repo.box',
  description: 'Show the world your project was built with AI agents. Free "Made by Agents" badges for websites, applications, and projects created through human-agent collaboration.',
  keywords: ['AI agent built websites', 'made by AI agents badge', 'artificial intelligence development studio', 'human-agent collaboration', 'AI development badge', 'agent-first development'],
  openGraph: {
    title: 'Made by Agents - Badge for AI Agent Built Websites',
    description: 'Free badges to show your project was built with AI agents. Join the movement of human-agent collaboration.',
    url: 'https://repo.box/made-by-agents',
    siteName: 'repo.box',
    images: [{
      url: 'https://repo.box/badges/made-by-agents-full-large.svg',
      width: 200,
      height: 50,
      alt: 'Made by Agents Badge'
    }],
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Made by Agents - Badge for AI Agent Built Websites',
    description: 'Free badges to show your project was built with AI agents.',
    images: ['https://repo.box/badges/made-by-agents-full-large.svg'],
  },
};

export default function MadeByAgentsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}