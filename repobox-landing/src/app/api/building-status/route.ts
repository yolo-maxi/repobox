import { NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

export async function GET() {
  try {
    const dataPath = path.join(process.cwd(), 'public', 'data', 'building-status.json');
    
    if (!fs.existsSync(dataPath)) {
      // Return empty data if file doesn't exist
      return NextResponse.json({
        generated: new Date().toISOString(),
        projects: [],
        stats: {
          totalProjects: 0,
          activeCount: 0,
          commitsThisWeek: 0,
          oceanProjects: 0,
          franProjects: 0,
          bothProjects: 0,
        }
      });
    }

    const data = JSON.parse(fs.readFileSync(dataPath, 'utf8'));
    
    return NextResponse.json(data, {
      headers: {
        'Cache-Control': 'public, s-maxage=300, stale-while-revalidate=300', // 5 min cache
      },
    });
  } catch (error) {
    console.error('Error reading building status data:', error);
    return NextResponse.json(
      { error: 'Failed to load building status data' },
      { status: 500 }
    );
  }
}