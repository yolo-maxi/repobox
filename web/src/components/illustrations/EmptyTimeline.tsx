'use client';

import React from 'react';

interface EmptyTimelineProps {
  className?: string;
}

export default function EmptyTimeline({ className = '' }: EmptyTimelineProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 80 80"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="No contribution activity illustration"
    >
      {/* Calendar grid background */}
      <rect
        x="15"
        y="20"
        width="50"
        height="35"
        rx="4"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.3"
      />
      
      {/* Calendar header */}
      <line
        x1="15"
        y1="28"
        x2="65"
        y2="28"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.3"
      />
      
      {/* Empty calendar squares */}
      {Array.from({ length: 35 }).map((_, i) => {
        const row = Math.floor(i / 7);
        const col = i % 7;
        const x = 18 + col * 6;
        const y = 31 + row * 4.5;
        
        return (
          <rect
            key={i}
            x={x}
            y={y}
            width="4"
            height="3"
            rx="0.5"
            fill="currentColor"
            opacity="0.1"
          />
        );
      })}
      
      {/* Dotted timeline indicator */}
      <line
        x1="25"
        y1="65"
        x2="55"
        y2="65"
        stroke="currentColor"
        strokeWidth="2"
        strokeDasharray="3,3"
        opacity="0.3"
      />
      
      {/* Time markers */}
      <circle cx="25" cy="65" r="2" fill="currentColor" opacity="0.2" />
      <circle cx="40" cy="65" r="2" fill="currentColor" opacity="0.2" />
      <circle cx="55" cy="65" r="2" fill="currentColor" opacity="0.2" />
      
      {/* Question mark indicator */}
      <circle
        cx="40"
        cy="12"
        r="8"
        stroke="currentColor"
        strokeWidth="2"
        strokeDasharray="2,2"
        opacity="0.3"
      />
      <path
        d="M37 9c0-1.5 1.5-3 3-3s3 1.5 3 3c0 2-3 2-3 4"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        opacity="0.4"
        fill="none"
      />
      <circle cx="40" cy="16" r="1" fill="currentColor" opacity="0.4" />
      
      {/* Floating clock icons */}
      <g opacity="0.2">
        <circle cx="70" cy="25" r="4" stroke="currentColor" strokeWidth="1" />
        <path d="M70 23v2l1 1" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
      </g>
      
      <g opacity="0.15">
        <circle cx="10" cy="45" r="3" stroke="currentColor" strokeWidth="1" />
        <path d="M10 44v1l1 1" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
      </g>
    </svg>
  );
}