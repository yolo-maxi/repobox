interface JazziconProps {
  address: string;
  size?: number;
  className?: string;
}

// Simple hash function to generate deterministic values from address
function hashCode(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash);
}

// Extract values from address bytes for deterministic generation
function extractValues(address: string) {
  // Remove 0x prefix and convert to lowercase
  const addr = address.slice(2).toLowerCase();
  
  // Extract different parts for different properties
  const hash1 = hashCode(addr.slice(0, 10));
  const hash2 = hashCode(addr.slice(10, 20));
  const hash3 = hashCode(addr.slice(20, 30));
  const hash4 = hashCode(addr.slice(30, 40));
  
  return { hash1, hash2, hash3, hash4 };
}

export default function Jazzicon({ address, size = 80, className = '' }: JazziconProps) {
  const { hash1, hash2, hash3, hash4 } = extractValues(address);
  
  // Generate 3 colors from the hash values
  const hue1 = hash1 % 360;
  const hue2 = (hash2 % 360 + 120) % 360; // Offset by 120 degrees
  const hue3 = (hash3 % 360 + 240) % 360; // Offset by 240 degrees
  
  const saturation = 65 + (hash4 % 20); // 65-85% saturation
  const lightness = 45 + (hash1 % 20); // 45-65% lightness
  
  const color1 = `hsl(${hue1}, ${saturation}%, ${lightness}%)`;
  const color2 = `hsl(${hue2}, ${saturation}%, ${lightness + 10}%)`;
  const color3 = `hsl(${hue3}, ${saturation}%, ${lightness - 10}%)`;
  
  // Generate shape parameters
  const radius1 = size * 0.3 + (hash2 % (size * 0.2));
  const radius2 = size * 0.2 + (hash3 % (size * 0.15));
  const radius3 = size * 0.15 + (hash4 % (size * 0.1));
  
  const x1 = size * 0.3 + (hash1 % (size * 0.4));
  const y1 = size * 0.3 + (hash2 % (size * 0.4));
  const x2 = size * 0.2 + (hash3 % (size * 0.6));
  const y2 = size * 0.2 + (hash4 % (size * 0.6));
  const x3 = size * 0.4 + (hash1 % (size * 0.2));
  const y3 = size * 0.4 + (hash2 % (size * 0.2));
  
  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      className={className}
    >
      {/* Background circle with gradient */}
      <defs>
        <radialGradient id={`bg-gradient-${address}`} cx="50%" cy="30%">
          <stop offset="0%" stopColor={color1} stopOpacity="0.8" />
          <stop offset="100%" stopColor={color2} stopOpacity="1" />
        </radialGradient>
      </defs>
      
      {/* Background */}
      <circle
        cx={size / 2}
        cy={size / 2}
        r={size / 2}
        fill={`url(#bg-gradient-${address})`}
      />
      
      {/* Shape 1 - Circle */}
      <circle
        cx={x1}
        cy={y1}
        r={radius1}
        fill={color2}
        opacity="0.7"
      />
      
      {/* Shape 2 - Rectangle */}
      <rect
        x={x2 - radius2 / 2}
        y={y2 - radius2 / 2}
        width={radius2}
        height={radius2}
        fill={color3}
        opacity="0.6"
        rx={radius2 * 0.2}
      />
      
      {/* Shape 3 - Small circle */}
      <circle
        cx={x3}
        cy={y3}
        r={radius3}
        fill={color1}
        opacity="0.8"
      />
      
      {/* Subtle border */}
      <circle
        cx={size / 2}
        cy={size / 2}
        r={size / 2 - 1}
        fill="none"
        stroke="rgba(255, 255, 255, 0.1)"
        strokeWidth="1"
      />
    </svg>
  );
}