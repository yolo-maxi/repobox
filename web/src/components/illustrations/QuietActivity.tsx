'use client';

import React from 'react';

interface QuietActivityProps {
  className?: string;
}

export default function QuietActivity({ className = '' }: QuietActivityProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 80 80"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="No activity illustration"
    >
      {/* Terminal window */}
      <rect
        x="15"
        y="20"
        width="50"
        height="35"
        rx="6"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.4"
      />
      
      {/* Terminal header */}
      <line
        x1="15"
        y1="28"
        x2="65"
        y2="28"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.4"
      />
      
      {/* Traffic light buttons */}
      <circle cx="22" cy="24" r="2" fill="currentColor" opacity="0.2" />
      <circle cx="30" cy="24" r="2" fill="currentColor" opacity="0.2" />
      <circle cx="38" cy="24" r="2" fill="currentColor" opacity="0.2" />
      
      {/* Sleeping cursor */}
      <rect
        x="20"
        y="35"
        width="2"
        height="12"
        fill="currentColor"
        opacity="0.3"
      >
        <animate
          attributeName="opacity"
          values="0.1;0.6;0.1"
          dur="2s"
          repeatCount="indefinite"
        />
      </rect>
      
      {/* Zzz sleep indicators */}
      <text
        x="45"
        y="30"
        fontSize="8"
        fill="currentColor"
        opacity="0.3"
        fontFamily="serif"
        style={{ fontStyle: 'italic' }}
      >
        z
      </text>
      <text
        x="50"
        y="26"
        fontSize="10"
        fill="currentColor"
        opacity="0.4"
        fontFamily="serif"
        style={{ fontStyle: 'italic' }}
      >
        z
      </text>
      <text
        x="56"
        y="21"
        fontSize="12"
        fill="currentColor"
        opacity="0.5"
        fontFamily="serif"
        style={{ fontStyle: 'italic' }}
      >
        Z
      </text>
      
      {/* Dimmed code lines */}
      <rect x="25" y="35" width="16" height="2" rx="1" fill="currentColor" opacity="0.15" />
      <rect x="25" y="40" width="24" height="2" rx="1" fill="currentColor" opacity="0.1" />
      <rect x="25" y="45" width="12" height="2" rx="1" fill="currentColor" opacity="0.1" />
      
      {/* Clock indicating quiet time */}
      <circle
        cx="40"
        cy="65"
        r="8"
        stroke="currentColor"
        strokeWidth="1.5"
        opacity="0.3"
      />
      <path
        d="M40 60v5l3 3"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        opacity="0.3"
      />
    </svg>
  );
}