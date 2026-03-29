import { NextRequest, NextResponse } from "next/server";

interface HireFormData {
  description: string;
  projectType: string;
  budget: string;
  email: string;
  timeline: string;
}

export async function POST(request: NextRequest) {
  try {
    const formData: HireFormData = await request.json();
    
    // Validate required fields
    const { description, projectType, budget, email, timeline } = formData;
    
    if (!description?.trim() || !projectType || !email?.trim() || !timeline) {
      return NextResponse.json(
        { error: "Missing required fields" },
        { status: 400 }
      );
    }
    
    // Validate email format
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
      return NextResponse.json(
        { error: "Invalid email format" },
        { status: 400 }
      );
    }
    
    // Validate description length
    if (description.length > 500) {
      return NextResponse.json(
        { error: "Description too long" },
        { status: 400 }
      );
    }
    
    // Format data for email
    const emailContent = `
New Project Inquiry - repo.box

PROJECT TYPE: ${projectType}
BUDGET: ${budget || "Not specified"}
TIMELINE: ${timeline}
EMAIL: ${email}

DESCRIPTION:
${description}

---
Submitted: ${new Date().toISOString()}
    `.trim();
    
    // Send email to enterprise@repo.box
    // Note: This would require email configuration in a real implementation
    // For now, we'll log the submission and return success
    console.log("=== NEW HIRE FORM SUBMISSION ===");
    console.log(emailContent);
    console.log("=================================");
    
    // In a real implementation, you'd:
    // 1. Send email to enterprise@repo.box using a service like SendGrid, SES, etc.
    // 2. Send auto-responder email to the user
    // 3. Track analytics event
    // 4. Store in database if needed
    
    // For now, simulate successful submission
    return NextResponse.json({ success: true });
    
  } catch (error) {
    console.error("Hire form submission error:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    );
  }
}