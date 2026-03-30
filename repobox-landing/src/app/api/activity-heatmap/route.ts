import { NextResponse } from 'next/server';
import { promises as fs } from 'fs';
import { join } from 'path';

interface HeatmapData {
  startDate: string;
  endDate: string;
  data: {
    [date: string]: {
      count: number;
      level: "empty" | "light" | "medium" | "intense";
      repos: string;
      repoCount: number;
    };
  };
}

let cachedData: HeatmapData | null = null;
let lastCacheTime = 0;
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes in milliseconds

export async function GET() {
  try {
    const now = Date.now();
    
    // Return cached data if still valid
    if (cachedData && (now - lastCacheTime < CACHE_DURATION)) {
      return NextResponse.json(cachedData);
    }

    // Load fresh data from static file
    const heatmapPath = join(process.cwd(), 'public', 'heatmap-data.json');
    const fileContent = await fs.readFile(heatmapPath, 'utf-8');
    const data = JSON.parse(fileContent) as HeatmapData;

    // Cache the data
    cachedData = data;
    lastCacheTime = now;

    return NextResponse.json(data);
  } catch (error) {
    console.error('Failed to load heatmap data:', error);
    return NextResponse.json(
      { error: 'Failed to load activity heatmap data' },
      { status: 500 }
    );
  }
}