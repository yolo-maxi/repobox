"use client";

export function MobileBackgroundGradient() {
  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        background: `
          linear-gradient(135deg, 
            rgba(79, 195, 247, 0.03) 0%,
            rgba(79, 195, 247, 0.01) 25%,
            rgba(120, 220, 255, 0.02) 50%,
            rgba(79, 195, 247, 0.01) 75%,
            rgba(79, 195, 247, 0.03) 100%
          ),
          radial-gradient(circle at 30% 20%, 
            rgba(79, 195, 247, 0.05) 0%,
            transparent 50%
          ),
          radial-gradient(circle at 70% 80%, 
            rgba(120, 220, 255, 0.04) 0%,
            transparent 50%
          )
        `,
        zIndex: 1,
        pointerEvents: "none",
      }}
    />
  );
}