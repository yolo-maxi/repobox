'use client';

import React from 'react';

interface AddressNotFoundProps {
  className?: string;
}

export default function AddressNotFound({ className = '' }: AddressNotFoundProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 80 80"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="Address not found illustration"
    >
      {/* Chain links */}
      <g stroke="currentColor" strokeWidth="3" opacity="0.4">
        {/* First link */}
        <circle cx="25" cy="35" r="8" fill="none" />
        <circle cx="25" cy="35" r="4" fill="none" />
        
        {/* Second link */}
        <circle cx="40" cy="35" r="8" fill="none" />
        <circle cx="40" cy="35" r="4" fill="none" />
        
        {/* Third link - broken */}
        <path
          d="M48 27a8 8 0 0 1 8 8"
          strokeLinecap="round"
          strokeDasharray="2,2"
        />
        <path
          d="M56 43a8 8 0 0 1-8-8"
          strokeLinecap="round"
          strokeDasharray="2,2"
        />
      </g>
      
      {/* Address format visualization */}
      <rect
        x="15"
        y="50"
        width="50"
        height="8"
        rx="4"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.3"
      />
      
      {/* 0x prefix */}
      <text
        x="20"
        y="56"
        fontSize="8"
        fill="currentColor"
        opacity="0.4"
        fontFamily="monospace"
      >
        0x
      </text>
      
      {/* Broken address segments */}
      <rect x="28" y="52" width="6" height="2" rx="1" fill="currentColor" opacity="0.3" />
      <rect x="36" y="52" width="4" height="2" rx="1" fill="currentColor" opacity="0.2" />
      <rect x="42" y="52" width="8" height="2" rx="1" fill="currentColor" opacity="0.1" />
      
      {/* Error indicator */}
      <circle
        cx="40"
        cy="20"
        r="8"
        stroke="currentColor"
        strokeWidth="2"
        opacity="0.5"
      />
      <path
        d="M36 16l8 8M44 16l-8 8"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        opacity="0.5"
      />
      
      {/* Invalid characters */}
      <text
        x="55"
        y="25"
        fontSize="10"
        fill="currentColor"
        opacity="0.3"
        fontFamily="monospace"
      >
        ?
      </text>
      
      <text
        x="12"
        y="35"
        fontSize="8"
        fill="currentColor"
        opacity="0.2"
        fontFamily="monospace"
      >
        @
      </text>
      
      <text
        x="65"
        y="45"
        fontSize="9"
        fill="currentColor"
        opacity="0.25"
        fontFamily="monospace"
      >
        #
      </text>
      
      {/* Network disconnection indicator */}
      <g opacity="0.2">
        <circle cx="70" cy="65" r="6" stroke="currentColor" strokeWidth="1" />
        <path
          d="M67 62l6 6M73 62l-6 6"
          stroke="currentColor"
          strokeWidth="1"
          strokeLinecap="round"
        />
      </g>
    </svg>
  );
}