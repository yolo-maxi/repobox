import { NextResponse } from "next/server";
import { getDocBySlug } from "@/lib/docs";

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ slug: string[] }> }
) {
  const { slug } = await params;
  const slugStr = slug.join("/");
  const doc = getDocBySlug(slugStr);

  if (!doc) {
    return new NextResponse("Not found", { status: 404 });
  }

  const filename = slug[slug.length - 1];
  return new NextResponse(doc.raw, {
    headers: {
      "Content-Type": "text/markdown; charset=utf-8",
      "Content-Disposition": `attachment; filename="${filename}.md"`,
    },
  });
}
