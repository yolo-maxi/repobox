'use client';

import { useState } from 'react';
import Link from 'next/link';

interface Badge {
  name: string;
  filename: string;
  width: number;
  height: number;
  description: string;
}

const badges: Badge[] = [
  // Light theme variants
  { name: 'Light Small', filename: 'made-by-agents-light-small.svg', width: 120, height: 30, description: 'Clean light theme, compact size' },
  { name: 'Light Medium', filename: 'made-by-agents-light-medium.svg', width: 150, height: 40, description: 'Standard light theme size' },
  { name: 'Light Large', filename: 'made-by-agents-light-large.svg', width: 200, height: 50, description: 'Prominent light theme display' },
  
  // Dark theme variants
  { name: 'Dark Small', filename: 'made-by-agents-dark-small.svg', width: 120, height: 30, description: 'Dark theme, compact size' },
  { name: 'Dark Medium', filename: 'made-by-agents-dark-medium.svg', width: 150, height: 40, description: 'Standard dark theme size' },
  { name: 'Dark Large', filename: 'made-by-agents-dark-large.svg', width: 200, height: 50, description: 'Prominent dark theme display' },
  
  // Minimal variants
  { name: 'Minimal Small', filename: 'made-by-agents-minimal-small.svg', width: 120, height: 30, description: 'Transparent, minimal styling' },
  { name: 'Minimal Medium', filename: 'made-by-agents-minimal-medium.svg', width: 150, height: 40, description: 'Clean, no-background design' },
  { name: 'Minimal Large', filename: 'made-by-agents-minimal-large.svg', width: 200, height: 50, description: 'Subtle, minimal presence' },
  
  // Full branding variants
  { name: 'Full Small', filename: 'made-by-agents-full-small.svg', width: 120, height: 30, description: 'With repo.box branding' },
  { name: 'Full Medium', filename: 'made-by-agents-full-medium.svg', width: 150, height: 40, description: 'Standard branded version' },
  { name: 'Full Large', filename: 'made-by-agents-full-large.svg', width: 200, height: 50, description: 'Full branding, prominent' }
];

export default function MadeByAgentsPage() {
  const [copiedCode, setCopiedCode] = useState<string | null>(null);
  
  const copyToClipboard = (text: string, type: string) => {
    navigator.clipboard.writeText(text).then(() => {
      setCopiedCode(type);
      setTimeout(() => setCopiedCode(null), 2000);
    });
  };
  
  const generateHtmlCode = (badge: Badge) => {
    const clickTrackingUrl = `https://repo.box/api/badge-clicks?variant=${badge.filename}&size=${badge.height}`;
    return `<a href="${clickTrackingUrl}">
  <img src="https://repo.box/badges/${badge.filename}" alt="Made by Agents" width="${badge.width}" height="${badge.height}" />
</a>`;
  };
  
  const generateMarkdownCode = (badge: Badge) => {
    const clickTrackingUrl = `https://repo.box/api/badge-clicks?variant=${badge.filename}&size=${badge.height}`;
    return `[![Made by Agents](https://repo.box/badges/${badge.filename})](${clickTrackingUrl})`;
  };

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white">
      {/* Header */}
      <header className="border-b border-gray-800 bg-black/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <Link href="/" className="text-xl font-bold text-[#10B981]">
              repo.box
            </Link>
            <nav className="flex items-center space-x-6">
              <Link href="/projects" className="text-gray-300 hover:text-white transition-colors">
                Projects
              </Link>
              <Link href="/blog" className="text-gray-300 hover:text-white transition-colors">
                Blog
              </Link>
            </nav>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h1 className="text-4xl sm:text-5xl font-bold mb-6">
            Made by Agents
          </h1>
          <p className="text-xl text-gray-400 mb-8 max-w-2xl mx-auto">
            Show the world your project was built with AI agents. Join the movement of human-agent collaboration.
          </p>
          <div className="flex justify-center">
            <img 
              src="/badges/made-by-agents-full-large.svg" 
              alt="Made by Agents - Large Badge"
              width="200"
              height="50"
              className="shadow-lg"
            />
          </div>
        </div>
      </section>

      {/* Badge Gallery */}
      <section className="py-16 bg-gray-900/30">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center mb-12">Choose Your Badge</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {badges.map((badge) => (
              <div key={badge.filename} className="bg-gray-800/50 rounded-lg p-6 border border-gray-700">
                <div className="flex justify-center mb-4 bg-white/5 rounded-lg py-8">
                  <img 
                    src={`/badges/${badge.filename}`} 
                    alt={badge.name}
                    width={badge.width}
                    height={badge.height}
                  />
                </div>
                <h3 className="text-lg font-semibold mb-2">{badge.name}</h3>
                <p className="text-gray-400 text-sm mb-4">{badge.description}</p>
                <p className="text-xs text-gray-500 mb-4">{badge.width} × {badge.height}px</p>
                
                {/* HTML Code */}
                <div className="mb-4">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium">HTML</span>
                    <button
                      onClick={() => copyToClipboard(generateHtmlCode(badge), `html-${badge.filename}`)}
                      className="text-xs bg-gray-700 hover:bg-gray-600 px-2 py-1 rounded transition-colors"
                    >
                      {copiedCode === `html-${badge.filename}` ? 'Copied!' : 'Copy'}
                    </button>
                  </div>
                  <code className="block bg-black/50 p-3 rounded text-xs text-green-400 font-mono whitespace-pre-wrap break-all">
                    {generateHtmlCode(badge)}
                  </code>
                </div>
                
                {/* Markdown Code */}
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium">Markdown</span>
                    <button
                      onClick={() => copyToClipboard(generateMarkdownCode(badge), `md-${badge.filename}`)}
                      className="text-xs bg-gray-700 hover:bg-gray-600 px-2 py-1 rounded transition-colors"
                    >
                      {copiedCode === `md-${badge.filename}` ? 'Copied!' : 'Copy'}
                    </button>
                  </div>
                  <code className="block bg-black/50 p-3 rounded text-xs text-green-400 font-mono whitespace-pre-wrap break-all">
                    {generateMarkdownCode(badge)}
                  </code>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Usage Examples */}
      <section className="py-16">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center mb-12">Usage Examples</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {/* Footer Placement */}
            <div className="bg-gray-800/30 rounded-lg p-6 border border-gray-700">
              <h3 className="text-lg font-semibold mb-4">Footer Placement</h3>
              <div className="bg-gray-900 rounded-lg p-4 text-sm">
                <div className="border-b border-gray-700 pb-4 mb-4">
                  <p className="text-gray-400">© 2026 YourProject</p>
                </div>
                <div className="flex justify-center">
                  <img src="/badges/made-by-agents-minimal-small.svg" alt="Made by Agents" width="120" height="30" />
                </div>
              </div>
              <p className="text-gray-400 text-sm mt-4">
                Subtle attribution in your site footer. Most common placement.
              </p>
            </div>
            
            {/* About Page */}
            <div className="bg-gray-800/30 rounded-lg p-6 border border-gray-700">
              <h3 className="text-lg font-semibold mb-4">About Page</h3>
              <div className="bg-gray-900 rounded-lg p-4 text-sm">
                <h4 className="font-semibold mb-2">How We Built This</h4>
                <p className="text-gray-400 mb-3">This project was created through human-agent collaboration...</p>
                <img src="/badges/made-by-agents-light-medium.svg" alt="Made by Agents" width="150" height="40" />
              </div>
              <p className="text-gray-400 text-sm mt-4">
                Prominent display with context about your development process.
              </p>
            </div>
            
            {/* Hero Section */}
            <div className="bg-gray-800/30 rounded-lg p-6 border border-gray-700">
              <h3 className="text-lg font-semibold mb-4">Hero Section</h3>
              <div className="bg-gray-900 rounded-lg p-4 text-sm">
                <h4 className="font-semibold mb-2">YourProject</h4>
                <p className="text-gray-400 mb-3">Revolutionary AI-powered app</p>
                <img src="/badges/made-by-agents-full-medium.svg" alt="Made by Agents" width="150" height="40" />
              </div>
              <p className="text-gray-400 text-sm mt-4">
                Bold statement for AI-first products. Shows innovation focus.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Documentation */}
      <section className="py-16 bg-gray-900/30">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center mb-12">Documentation</h2>
          
          <div className="prose prose-invert max-w-none">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div>
                <h3 className="text-xl font-semibold mb-4">Installation</h3>
                <div className="space-y-4 text-gray-300">
                  <div>
                    <p className="font-medium mb-2">1. Choose a badge variant</p>
                    <p className="text-sm">Select from light, dark, minimal, or full branding options in various sizes.</p>
                  </div>
                  <div>
                    <p className="font-medium mb-2">2. Copy embed code</p>
                    <p className="text-sm">Use the HTML or Markdown code snippets provided above.</p>
                  </div>
                  <div>
                    <p className="font-medium mb-2">3. Add to your site</p>
                    <p className="text-sm">Paste the code in your footer, about page, or anywhere you want to show the badge.</p>
                  </div>
                </div>
              </div>
              
              <div>
                <h3 className="text-xl font-semibold mb-4">Best Practices</h3>
                <div className="space-y-4 text-gray-300">
                  <div>
                    <p className="font-medium mb-2">Badge placement</p>
                    <p className="text-sm">Footer for subtle attribution, hero/about for prominent display.</p>
                  </div>
                  <div>
                    <p className="font-medium mb-2">Theme matching</p>
                    <p className="text-sm">Use light badges on light backgrounds, dark on dark, minimal for any theme.</p>
                  </div>
                  <div>
                    <p className="font-medium mb-2">Size selection</p>
                    <p className="text-sm">Small for compact spaces, medium for standard, large for emphasis.</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="mt-12 p-6 bg-gray-800/50 rounded-lg border border-gray-700">
              <h3 className="text-xl font-semibold mb-4">License & Attribution</h3>
              <p className="text-gray-300 mb-4">
                These badges are released under the MIT License. You're free to use them on any project 
                built with AI agents. No registration required, no tracking beyond click analytics.
              </p>
              <p className="text-gray-300">
                The badges link back to repo.box to help others discover agent-built projects and join the movement.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-16">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-3xl font-bold mb-6">Ready to Show Your Agent-Built Project?</h2>
          <p className="text-xl text-gray-400 mb-8">
            Join the growing movement of human-agent collaboration in software development.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link 
              href="/hire"
              className="bg-[#10B981] hover:bg-[#0ea574] text-black font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              Build with Our Agents
            </Link>
            <Link 
              href="/agents"
              className="bg-gray-800 hover:bg-gray-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              Meet Our Agents
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-gray-800 bg-black/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="py-8 flex flex-col md:flex-row justify-between items-center">
            <div className="flex items-center space-x-4">
              <Link href="/" className="text-lg font-bold text-[#10B981]">
                repo.box
              </Link>
              <span className="text-gray-500">×</span>
              <img src="/badges/made-by-agents-minimal-small.svg" alt="Made by Agents" width="120" height="30" />
            </div>
            <div className="mt-4 md:mt-0">
              <Link href="/trust" className="text-gray-400 hover:text-white transition-colors">
                Security & Trust
              </Link>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}