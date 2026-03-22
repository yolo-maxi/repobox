'use client';

import React from 'react';

interface NoDiffProps {
  className?: string;
}

export default function NoDiff({ className = '' }: NoDiffProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 80 80"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="No diff available illustration"
    >
      {/* Document outline */}
      <path
        d="M25 15h25l10 10v40c0 2-1.8 4-4 4H29c-2.2 0-4-1.8-4-4V15z"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.4"
      />
      
      {/* Folded corner */}
      <path
        d="M50 15v10h10"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.3"
      />
      
      {/* Binary/hex content representation */}
      <g opacity="0.3" fontFamily="monospace" fontSize="6">
        <text x="30" y="35" fill="currentColor">01101</text>
        <text x="30" y="42" fill="currentColor">11010</text>
        <text x="30" y="49" fill="currentColor">00101</text>
        <text x="45" y="35" fill="currentColor">AF2E</text>
        <text x="45" y="42" fill="currentColor">B1C3</text>
        <text x="45" y="49" fill="currentColor">9F7A</text>
      </g>
      
      {/* Question mark overlay */}
      <circle
        cx="40"
        cy="40"
        r="15"
        fill="rgba(255,255,255,0.1)"
        stroke="currentColor"
        strokeWidth="2"
        strokeDasharray="3,3"
        opacity="0.5"
      />
      
      <g transform="translate(40,40)" opacity="0.6">
        <path
          d="M-3 -6c0-2 1.5-4 4-4s4 2 4 4c0 2-4 2-4 5"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          fill="none"
        />
        <circle cx="1" cy="4" r="1" fill="currentColor" />
      </g>
      
      {/* File type indicators */}
      <circle cx="15" cy="25" r="2" fill="currentColor" opacity="0.2" />
      <rect x="12" y="30" width="6" height="1" rx="0.5" fill="currentColor" opacity="0.2" />
      
      <circle cx="65" cy="55" r="2" fill="currentColor" opacity="0.15" />
      <rect x="62" y="60" width="6" height="1" rx="0.5" fill="currentColor" opacity="0.15" />
      
      {/* Size warning indicator */}
      <path
        d="M20 65l3-6 3 6M22 62h2"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        opacity="0.3"
      />
    </svg>
  );
}