import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';
import matter from 'gray-matter';
import { marked } from 'marked';
import Link from 'next/link';
import { notFound } from 'next/navigation';

async function getBlogPost(slug: string) {
  try {
    const fullPath = join(process.cwd(), 'posts', `${slug}.md`);
    const fileContents = readFileSync(fullPath, 'utf8');
    const { data, content } = matter(fileContents);
    
    const html = marked(content);
    
    return {
      slug,
      title: data.title || slug,
      date: data.date || '',
      content: html,
      ...data,
    };
  } catch {
    return null;
  }
}

export async function generateStaticParams() {
  const postsDirectory = join(process.cwd(), 'posts');
  const filenames = readdirSync(postsDirectory).filter(name => name.endsWith('.md'));
  
  return filenames.map((name) => ({
    slug: name.replace(/\.md$/, ''),
  }));
}

export default async function BlogPost({ params }: { params: { slug: string } }) {
  const post = await getBlogPost(params.slug);

  if (!post) {
    notFound();
  }

  return (
    <>
      <div style={{ maxWidth: 720, margin: '0 auto', padding: '80px 40px 100px' }}>
        <header style={{ marginBottom: 60 }}>
          <Link href="/blog" style={{ 
            color: 'var(--bp-accent)', 
            textDecoration: 'none',
            fontSize: '0.9rem',
            marginBottom: 20,
            display: 'inline-block'
          }}>
            ← Blog
          </Link>
          <h1 style={{ 
            fontSize: '2.5rem', 
            color: 'var(--bp-heading)', 
            lineHeight: 1.2,
            marginBottom: 12,
            margin: 0
          }}>
            {post.title}
          </h1>
          {post.date && (
            <time style={{ 
              color: 'var(--bp-dim)', 
              fontSize: '1rem' 
            }}>
              {typeof post.date === 'string' ? post.date : post.date.toString()}
            </time>
          )}
        </header>

        <article 
          className="blog-content"
          style={{ 
            lineHeight: 1.7,
            fontSize: '1.1rem'
          }}
          dangerouslySetInnerHTML={{ __html: post.content }}
        />
      </div>
    </>
  );
}