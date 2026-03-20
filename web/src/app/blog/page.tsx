import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';
import matter from 'gray-matter';
import Link from 'next/link';

function getBlogPosts() {
  const postsDirectory = join(process.cwd(), 'posts');
  const filenames = readdirSync(postsDirectory).filter(name => name.endsWith('.md'));
  
  const posts = filenames.map((name) => {
    const fullPath = join(postsDirectory, name);
    const fileContents = readFileSync(fullPath, 'utf8');
    const { data, content } = matter(fileContents);
    const slug = name.replace(/\.md$/, '');
    
    return {
      slug,
      title: data.title || slug,
      date: data.date || '',
      excerpt: data.excerpt || content.substring(0, 200) + '...',
      ...data,
    };
  });

  return posts.sort((a, b) => (a.date > b.date ? -1 : 1));
}

export default function BlogPage() {
  const posts = getBlogPosts();

  return (
    <>
      <div style={{ maxWidth: 720, margin: '0 auto', padding: '80px 40px 100px' }}>
        <header style={{ marginBottom: 60 }}>
          <Link href="/" style={{ 
            color: 'var(--bp-accent)', 
            textDecoration: 'none',
            fontSize: '0.9rem',
            marginBottom: 20,
            display: 'inline-block'
          }}>
            ← repo.box
          </Link>
          <h1 style={{ 
            fontSize: '2rem', 
            color: 'var(--bp-heading)', 
            marginBottom: 12,
            margin: 0
          }}>
            Blog
          </h1>
          <p style={{ color: 'var(--bp-dim)', fontSize: '1.1rem' }}>
            Building the future of AI-safe git workflows
          </p>
        </header>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 40 }}>
          {posts.map(post => (
            <article key={post.slug} style={{ 
              borderBottom: '1px solid var(--bp-border)', 
              paddingBottom: 40 
            }}>
              <header style={{ marginBottom: 16 }}>
                <h2 style={{ 
                  fontSize: '1.4rem', 
                  margin: 0,
                  marginBottom: 8
                }}>
                  <Link 
                    href={`/blog/${post.slug}`}
                    style={{ 
                      color: 'var(--bp-heading)', 
                      textDecoration: 'none' 
                    }}
                  >
                    {post.title}
                  </Link>
                </h2>
                {post.date && (
                  <time style={{ 
                    color: 'var(--bp-dim)', 
                    fontSize: '0.9rem' 
                  }}>
                    {typeof post.date === 'string' ? post.date : post.date.toString()}
                  </time>
                )}
              </header>
              
              <p style={{ 
                color: 'var(--bp-text)', 
                lineHeight: 1.6,
                margin: 0
              }}>
                {post.excerpt}
              </p>
            </article>
          ))}
        </div>
      </div>
    </>
  );
}