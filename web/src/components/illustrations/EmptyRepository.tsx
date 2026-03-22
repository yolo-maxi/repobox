'use client';

import React from 'react';

interface EmptyRepositoryProps {
  className?: string;
}

export default function EmptyRepository({ className = '' }: EmptyRepositoryProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 80 80"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="No repositories illustration"
    >
      {/* Background container */}
      <rect 
        x="12" 
        y="20" 
        width="56" 
        height="40" 
        rx="8" 
        stroke="currentColor" 
        strokeWidth="2" 
        strokeDasharray="4,4" 
        opacity="0.4"
      />
      
      {/* Git folder icon */}
      <path
        d="M20 28h8l4 6h24v20c0 2.2-1.8 4-4 4H24c-2.2 0-4-1.8-4-4V28z"
        stroke="currentColor"
        strokeWidth="2"
        fill="none"
        opacity="0.6"
      />
      
      {/* Git branch symbol */}
      <circle 
        cx="32" 
        cy="38" 
        r="3" 
        stroke="currentColor" 
        strokeWidth="2" 
        fill="none"
        opacity="0.5"
      />
      <path
        d="M32 35V25M35 38l8-8"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        opacity="0.5"
      />
      <circle 
        cx="46" 
        cy="27" 
        r="3" 
        stroke="currentColor" 
        strokeWidth="2" 
        fill="none"
        opacity="0.5"
      />
      
      {/* Floating code elements */}
      <text
        x="58"
        y="18"
        fontSize="10"
        fill="currentColor"
        opacity="0.3"
        fontFamily="monospace"
      >
        &lt;/&gt;
      </text>
      
      <circle
        cx="16"
        cy="15"
        r="2"
        fill="currentColor"
        opacity="0.2"
      />
      
      <rect
        x="22"
        y="65"
        width="8"
        height="2"
        rx="1"
        fill="currentColor"
        opacity="0.3"
      />
      
      <path
        d="M60 65h4m2 0h4"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        opacity="0.25"
      />
    </svg>
  );
}