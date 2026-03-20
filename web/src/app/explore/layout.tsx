export default function ExploreLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="explore-bg min-h-screen">
      {/* Glass panel */}
      <div className="min-h-screen flex justify-center px-3 py-6 md:px-6 lg:px-10">
        <div className="w-full max-w-6xl">
          <div className="glass-panel rounded-2xl min-h-[calc(100vh-3rem)] overflow-hidden">
            {children}
          </div>
        </div>
      </div>
    </div>
  );
}
