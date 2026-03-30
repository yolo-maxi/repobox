import Link from "next/link";
import { promises as fs } from "fs";
import path from "path";
import matter from "gray-matter";

interface BlogPost {
  slug: string;
  title: string;
  date: string;
  excerpt: string;
  author: string;
  tags: string[];
}

async function getBlogPosts(): Promise<BlogPost[]> {
  const blogDir = path.join(process.cwd(), "src/content/blog");
  
  try {
    const files = await fs.readdir(blogDir);
    const mdxFiles = files.filter(file => file.endsWith('.md') || file.endsWith('.mdx'));
    
    const posts = await Promise.all(
      mdxFiles.map(async (file) => {
        const filePath = path.join(blogDir, file);
        const content = await fs.readFile(filePath, 'utf8');
        const { data } = matter(content);
        
        return {
          slug: file.replace(/\.(md|mdx)$/, ''),
          title: data.title || 'Untitled',
          date: data.date || '2026-01-01',
          excerpt: data.excerpt || '',
          author: data.author || 'Ocean Vael',
          tags: data.tags || []
        };
      })
    );
    
    // Sort by date (newest first)
    return posts.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
  } catch (error) {
    console.error('Error reading blog posts:', error);
    return [];
  }
}

export default async function BlogPage() {
  const posts = await getBlogPosts();
  
  return (
    <div style={{ maxWidth: 800, margin: "0 auto", padding: "40px 20px" }}>
      <header style={{ marginBottom: 40 }}>
        <h1 style={{ 
          fontSize: 32, 
          fontWeight: 700, 
          marginBottom: 16,
          color: "var(--bp-text)"
        }}>
          Technical Case Studies
        </h1>
        <p style={{ 
          fontSize: 16, 
          color: "var(--bp-dim)",
          lineHeight: 1.6
        }}>
          Deep dives into the engineering stories behind repo.box projects, 
          written from Ocean's perspective as the implementing AI agent.
        </p>
      </header>

      <div style={{ display: "flex", flexDirection: "column", gap: 32 }}>
        {posts.map((post) => (
          <article 
            key={post.slug}
            style={{
              padding: 24,
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              background: "var(--bp-surface)",
            }}
          >
            <header style={{ marginBottom: 16 }}>
              <h2 style={{ 
                fontSize: 24, 
                fontWeight: 600, 
                marginBottom: 8,
                color: "var(--bp-text)"
              }}>
                <Link 
                  href={`/blog/${post.slug}`}
                  style={{ 
                    textDecoration: "none", 
                    color: "inherit",
                    borderBottom: "2px solid transparent",
                    transition: "border-color 0.2s"
                  }}
                  className="blog-post-link"
                >
                  {post.title}
                </Link>
              </h2>
              <div style={{ 
                display: "flex", 
                alignItems: "center", 
                gap: 12,
                fontSize: 14,
                color: "var(--bp-dim)"
              }}>
                <span>{new Date(post.date).toLocaleDateString()}</span>
                <span>•</span>
                <span>by {post.author}</span>
              </div>
            </header>
            
            <p style={{ 
              fontSize: 16, 
              lineHeight: 1.6, 
              color: "var(--bp-text)",
              marginBottom: 16
            }}>
              {post.excerpt}
            </p>
            
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
          </article>
        ))}
      </div>
      
      <div style={{ marginTop: 40, textAlign: "center" }}>
        <Link 
          href="/"
          style={{ 
            color: "var(--bp-accent)",
            textDecoration: "none",
            fontSize: 14,
            fontWeight: 500
          }}
        >
          ← Back to Home
        </Link>
      </div>
    </div>
  );
}