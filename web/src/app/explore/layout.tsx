import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function ExploreLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="relative min-h-screen">
      {/* Jellyfish/manta ray background at edges */}
      <BackgroundCanvas />
      
      {/* Glass panel overlay */}
      <div className="relative z-10 min-h-screen flex justify-center px-4 py-8 md:px-8">
        <div className="w-full max-w-6xl">
          <div className="glass-panel rounded-2xl min-h-[calc(100vh-4rem)] overflow-hidden">
            {children}
          </div>
        </div>
      </div>
    </div>
  );
}
