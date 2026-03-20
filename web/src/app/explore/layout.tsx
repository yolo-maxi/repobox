export default function ExploreLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="explore-container">
      {children}
    </div>
  );
}