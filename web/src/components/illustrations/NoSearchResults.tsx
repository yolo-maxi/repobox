'use client';

import React from 'react';

interface NoSearchResultsProps {
  className?: string;
}

export default function NoSearchResults({ className = '' }: NoSearchResultsProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 80 80"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="No search results illustration"
    >
      {/* Magnifying glass */}
      <circle
        cx="35"
        cy="35"
        r="18"
        stroke="currentColor"
        strokeWidth="3"
        opacity="0.4"
      />
      <path
        d="M49 49l15 15"
        stroke="currentColor"
        strokeWidth="3"
        strokeLinecap="round"
        opacity="0.4"
      />
      
      {/* Empty search area */}
      <circle
        cx="35"
        cy="35"
        r="12"
        stroke="currentColor"
        strokeWidth="2"
        strokeDasharray="4,4"
        opacity="0.3"
      />
      
      {/* Search query visualization */}
      <rect
        x="15"
        y="10"
        width="30"
        height="6"
        rx="3"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.3"
      />
      <rect
        x="18"
        y="12"
        width="8"
        height="2"
        rx="1"
        fill="currentColor"
        opacity="0.4"
      />
      
      {/* No results indicator */}
      <g opacity="0.5">
        <circle cx="35" cy="35" r="8" fill="none" />
        <path
          d="M30 30l10 10M40 30l-10 10"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          opacity="0.4"
        />
      </g>
      
      {/* Floating search terms */}
      <text
        x="55"
        y="25"
        fontSize="8"
        fill="currentColor"
        opacity="0.2"
        fontFamily="monospace"
      >
        repo
      </text>
      
      <text
        x="10"
        y="60"
        fontSize="7"
        fill="currentColor"
        opacity="0.15"
        fontFamily="monospace"
      >
        git
      </text>
      
      <text
        x="60"
        y="70"
        fontSize="6"
        fill="currentColor"
        opacity="0.1"
        fontFamily="monospace"
      >
        code
      </text>
      
      {/* Search suggestions dots */}
      <circle cx="20" cy="70" r="1.5" fill="currentColor" opacity="0.2" />
      <circle cx="26" cy="70" r="1.5" fill="currentColor" opacity="0.15" />
      <circle cx="32" cy="70" r="1.5" fill="currentColor" opacity="0.1" />
    </svg>
  );
}