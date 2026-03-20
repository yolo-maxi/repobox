import fs from "fs";
import path from "path";
import matter from "gray-matter";
import { marked } from "marked";

const DOCS_DIR = path.join(process.cwd(), "content", "docs");
const MANIFEST_PATH = path.join(DOCS_DIR, "manifest.json");

export interface DocPage {
  slug: string;
  title: string;
  description: string;
  html: string;
  raw: string;
}

export interface ManifestSection {
  title: string;
  items: { title: string; slug: string }[];
}

export interface Manifest {
  sections: ManifestSection[];
}

export function getManifest(): Manifest {
  const raw = fs.readFileSync(MANIFEST_PATH, "utf8");
  return JSON.parse(raw);
}

export function getDocBySlug(slug: string): DocPage | null {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
  if (!fs.existsSync(filePath)) return null;

  const raw = fs.readFileSync(filePath, "utf8");
  const { data, content } = matter(raw);

  return {
    slug,
    title: data.title || slug,
    description: data.description || "",
    html: marked(content) as string,
    raw: content,
  };
}

export function getAllSlugs(): string[] {
  const manifest = getManifest();
  return manifest.sections.flatMap((s) => s.items.map((i) => i.slug));
}
