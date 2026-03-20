"use client";

import { useState, useEffect } from "react";

export function useIsMobile() {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const checkMobile = () => {
      // Check viewport width (mobile breakpoint)
      const isSmallScreen = window.innerWidth < 768;
      
      // Check user agent for mobile/tablet devices
      const isMobileUserAgent = /Mobi|Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
      
      // Check for low-end devices based on hardware concurrency
      const isLowEndDevice = !!(navigator.hardwareConcurrency && navigator.hardwareConcurrency <= 4);
      
      // Check for reduced motion preference (often enabled on lower-end devices)
      const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      
      // Consider device mobile if any of these conditions are true
      setIsMobile(isSmallScreen || isMobileUserAgent || (isLowEndDevice && prefersReducedMotion));
    };

    checkMobile();

    // Listen for window resize to handle orientation changes
    window.addEventListener('resize', checkMobile);
    
    // Listen for media query changes
    const mediaQuery = window.matchMedia('(max-width: 767px)');
    mediaQuery.addEventListener('change', checkMobile);

    return () => {
      window.removeEventListener('resize', checkMobile);
      mediaQuery.removeEventListener('change', checkMobile);
    };
  }, []);

  return isMobile;
}