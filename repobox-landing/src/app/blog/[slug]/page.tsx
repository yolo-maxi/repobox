import { promises as fs } from "fs";
import path from "path";
import matter from "gray-matter";
import { remark } from "remark";
import html from "remark-html";
import { notFound } from "next/navigation";
import Link from "next/link";

interface BlogPostData {
  title: string;
  date: string;
  excerpt: string;
  author: string;
  tags: string[];
  content: string;
}

async function getBlogPost(slug: string): Promise<BlogPostData | null> {
  const blogDir = path.join(process.cwd(), "src/content/blog");
  const filePath = path.join(blogDir, `${slug}.md`);
  
  try {
    const fileContent = await fs.readFile(filePath, 'utf8');
    const { data, content } = matter(fileContent);
    
    // Convert markdown to HTML
    const processedContent = await remark()
      .use(html, { sanitize: false })
      .process(content);
    
    return {
      title: data.title || 'Untitled',
      date: data.date || '2026-01-01',
      excerpt: data.excerpt || '',
      author: data.author || 'Ocean Vael',
      tags: data.tags || [],
      content: processedContent.toString()
    };
  } catch (error) {
    console.error(`Error reading blog post ${slug}:`, error);
    return null;
  }
}

export async function generateStaticParams() {
  const blogDir = path.join(process.cwd(), "src/content/blog");
  
  try {
    const files = await fs.readdir(blogDir);
    const mdFiles = files.filter(file => file.endsWith('.md'));
    
    return mdFiles.map((file) => ({
      slug: file.replace(/\.md$/, ''),
    }));
  } catch (error) {
    return [];
  }
}

export async function generateMetadata({ params }: { params: { slug: string } }) {
  const post = await getBlogPost(params.slug);
  
  if (!post) {
    return {
      title: 'Post Not Found - repo.box',
    };
  }
  
  return {
    title: `${post.title} - repo.box`,
    description: post.excerpt,
    openGraph: {
      title: post.title,
      description: post.excerpt,
      type: 'article',
      authors: [post.author],
      publishedTime: post.date,
      tags: post.tags,
    },
    twitter: {
      card: 'summary_large_image',
      title: post.title,
      description: post.excerpt,
    },
  };
}

export default async function BlogPostPage({ params }: { params: { slug: string } }) {
  const post = await getBlogPost(params.slug);
  
  if (!post) {
    notFound();
  }
  
  return (
    <div style={{ maxWidth: 800, margin: "0 auto", padding: "40px 20px" }}>
      {/* Navigation */}
      <div style={{ marginBottom: 32 }}>
        <Link 
          href="/blog"
          style={{ 
            color: "var(--bp-accent)",
            textDecoration: "none",
            fontSize: 14,
            fontWeight: 500
          }}
        >
          ← Back to Case Studies
        </Link>
      </div>

      {/* Article Header */}
      <header style={{ marginBottom: 40 }}>
        <h1 style={{ 
          fontSize: 36, 
          fontWeight: 700, 
          lineHeight: 1.2,
          marginBottom: 16,
          color: "var(--bp-text)"
        }}>
          {post.title}
        </h1>
        
        <div style={{ 
          display: "flex", 
          alignItems: "center", 
          gap: 16,
          fontSize: 14,
          color: "var(--bp-dim)",
          marginBottom: 16
        }}>
          <span>{new Date(post.date).toLocaleDateString('en-US', { 
            year: 'numeric', 
            month: 'long', 
            day: 'numeric' 
          })}</span>
          <span>•</span>
          <span>by {post.author}</span>
          <span>•</span>
          <span>~{Math.ceil(post.content.length / 1000)} min read</span>
        </div>
        
        {post.tags.length > 0 && (
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            {post.tags.map((tag) => (
              <span
                key={tag}
                style={{
                  fontSize: 12,
                  padding: "4px 8px",
                  backgroundColor: "rgba(79,195,247,0.15)",
                  color: "var(--bp-accent)",
                  borderRadius: 4,
                  fontWeight: 500
                }}
              >
                {tag}
              </span>
            ))}
          </div>
        )}
      </header>

      {/* Article Content */}
      <article 
        style={{
          fontSize: 16,
          lineHeight: 1.7,
          color: "var(--bp-text)",
        }}
        className="blog-content"
        dangerouslySetInnerHTML={{ __html: post.content }}
      />

      {/* Call to Action */}
      <footer style={{ 
        marginTop: 60, 
        padding: 32,
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
        background: "var(--bp-surface)",
        textAlign: "center"
      }}>
        <h3 style={{ 
          fontSize: 20, 
          fontWeight: 600, 
          marginBottom: 16,
          color: "var(--bp-text)"
        }}>
          Ready to build something similar?
        </h3>
        <p style={{ 
          fontSize: 16, 
          color: "var(--bp-dim)", 
          marginBottom: 24
        }}>
          Ocean and the repo.box team are available for consulting on AI agent architecture, 
          Web3 integrations, and complex system design.
        </p>
        <div style={{ display: "flex", gap: 16, justifyContent: "center" }}>
          <Link
            href="/hire"
            style={{
              display: "inline-block",
              padding: "12px 24px",
              backgroundColor: "var(--bp-accent)",
              color: "white",
              textDecoration: "none",
              borderRadius: 6,
              fontWeight: 500,
              fontSize: 14
            }}
          >
            Book a Consultation
          </Link>
          <Link
            href="/trust"
            style={{
              display: "inline-block",
              padding: "12px 24px",
              border: "1px solid var(--bp-border)",
              color: "var(--bp-text)",
              textDecoration: "none",
              borderRadius: 6,
              fontWeight: 500,
              fontSize: 14
            }}
          >
            View Our Work
          </Link>
        </div>
      </footer>
    </div>
  );
}