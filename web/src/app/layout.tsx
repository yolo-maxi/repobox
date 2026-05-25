import type { Metadata } from "next";
import { JetBrains_Mono } from "next/font/google";
import "./globals.css";

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
});

export const metadata: Metadata = {
  metadataBase: new URL("https://repo.box"),
  title: "repo.box",
  description: "An independent team building cool ideas with cool people.",
  openGraph: {
    title: "repo.box",
    description: "An independent team building cool ideas with cool people.",
    url: "https://repo.box",
    siteName: "repo.box",
    images: [{ url: "/og/repo-box-share.jpg", width: 1200, height: 630 }],
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "repo.box",
    description: "An independent team building cool ideas with cool people.",
    images: ["/og/repo-box-share.jpg"],
  },
  icons: { icon: "/favicon.svg" },
  alternates: {
    types: { "application/rss+xml": "/feed.xml" },
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={jetbrainsMono.variable}>
      <body>{children}</body>
    </html>
  );
}