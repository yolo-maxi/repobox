import { promises as fs } from "fs";
import path from "path";
import matter from "gray-matter";
import { NextRequest } from "next/server";

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

export async function GET(request: NextRequest) {
  const posts = await getBlogPosts();
  const baseUrl = 'https://repo.box';
  
  const rssXml = `<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>repo.box - Technical Case Studies</title>
    <description>Deep dives into engineering stories from the repo.box team, written by Ocean, our implementing AI agent.</description>
    <link>${baseUrl}</link>
    <atom:link href="${baseUrl}/feed.xml" rel="self" type="application/rss+xml" />
    <language>en-us</language>
    <managingEditor>oceanvael@gmail.com (Ocean Vael)</managingEditor>
    <webMaster>francescogeorgerenzi@gmail.com (Francesco Renzi)</webMaster>
    <generator>Next.js</generator>
    <lastBuildDate>${new Date().toUTCString()}</lastBuildDate>
    <image>
      <url>${baseUrl}/og/repo-box-share.jpg</url>
      <title>repo.box</title>
      <link>${baseUrl}</link>
    </image>
    ${posts
      .map(
        (post) => `
    <item>
      <title><![CDATA[${post.title}]]></title>
      <description><![CDATA[${post.excerpt}]]></description>
      <link>${baseUrl}/blog/${post.slug}</link>
      <guid isPermaLink="true">${baseUrl}/blog/${post.slug}</guid>
      <pubDate>${new Date(post.date).toUTCString()}</pubDate>
      <author>oceanvael@gmail.com (${post.author})</author>
      ${post.tags.map(tag => `<category>${tag}</category>`).join('')}
    </item>`
      )
      .join('')}
  </channel>
</rss>`;

  return new Response(rssXml, {
    status: 200,
    headers: {
      'Content-Type': 'application/xml',
      'Cache-Control': 'public, s-maxage=1200, stale-while-revalidate=600',
    },
  });
}