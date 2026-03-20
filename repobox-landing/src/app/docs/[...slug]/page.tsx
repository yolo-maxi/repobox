import { notFound } from "next/navigation";
import { getDocBySlug, getManifest, getAllSlugs } from "@/lib/docs";
import { DocsSidebar } from "@/components/docs/sidebar";
import { DocsContent } from "@/components/docs/content";
import type { Metadata } from "next";

interface Props {
  params: Promise<{ slug: string[] }>;
}

export async function generateStaticParams() {
  const slugs = getAllSlugs();
  return slugs.map((s) => ({ slug: s.split("/") }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const slugStr = slug.join("/");
  const doc = getDocBySlug(slugStr);
  if (!doc) return {};
  return {
    title: `${doc.title} — repo.box docs`,
    description: doc.description,
  };
}

export default async function DocsPage({ params }: Props) {
  const { slug } = await params;
  const slugStr = slug.join("/");
  const doc = getDocBySlug(slugStr);
  if (!doc) notFound();

  const manifest = getManifest();

  return (
    <div className="min-h-screen" style={{ background: "var(--bp-bg)" }}>
      <div className="docs-layout">
        <DocsSidebar manifest={manifest} currentSlug={slugStr} />
        <DocsContent doc={doc} />
      </div>
    </div>
  );
}
