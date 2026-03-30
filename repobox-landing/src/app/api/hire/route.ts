import { NextRequest, NextResponse } from 'next/server';

interface HireFormData {
  description: string;
  projectType: string;
  budget: string;
  timeline: string;
  email: string;
}

export async function POST(request: NextRequest) {
  try {
    const data: HireFormData = await request.json();
    
    // Validate required fields
    if (!data.description || !data.projectType || !data.timeline || !data.email) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    // Validate email format
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(data.email)) {
      return NextResponse.json({ error: 'Invalid email format' }, { status: 400 });
    }

    // Validate description length
    if (data.description.length > 500) {
      return NextResponse.json({ error: 'Description too long' }, { status: 400 });
    }

    // Format email content
    const emailSubject = `New Hire Request: ${data.projectType} Project`;
    const emailBody = `
New hire request received from repo.box/hire

Project Details:
- Type: ${data.projectType}
- Budget: ${data.budget || 'Not specified'}
- Timeline: ${data.timeline}
- Contact: ${data.email}

Description:
${data.description}

---
Submitted: ${new Date().toISOString()}
Source: repo.box/hire
    `.trim();

    // Send email to enterprise@repo.box
    // Note: This would typically use a service like SendGrid, Resend, or nodemailer
    // For now, we'll log it and save to a file for manual processing
    
    console.log('Hire form submission:', {
      subject: emailSubject,
      body: emailBody,
      timestamp: new Date().toISOString()
    });

    // In a production setup, you would integrate with an email service here
    // Example with a hypothetical email service:
    /*
    await sendEmail({
      to: 'enterprise@repo.box',
      subject: emailSubject,
      text: emailBody,
      replyTo: data.email
    });
    */

    // Send auto-responder to the user
    const autoResponderSubject = 'Thank you for your project inquiry - repo.box';
    const autoResponderBody = `
Hi there,

Thank you for reaching out about your ${data.projectType.toLowerCase()} project!

We've received your request and our team will review it within 24 hours. Someone will reach out to discuss your project in detail and provide next steps.

Project Summary:
- Type: ${data.projectType}
- Timeline: ${data.timeline}
${data.budget ? `- Budget: ${data.budget}` : ''}

In the meantime, feel free to explore our portfolio at repo.box to see examples of our work.

Best regards,
The repo.box Team

---
This is an automated response. Please don't reply to this email.
    `.trim();

    console.log('Auto-responder email:', {
      to: data.email,
      subject: autoResponderSubject,
      body: autoResponderBody,
      timestamp: new Date().toISOString()
    });

    // Save submission to a local file for manual processing
    // This is a fallback until proper email integration is set up
    const fs = require('fs').promises;
    const path = require('path');
    
    const submissionData = {
      timestamp: new Date().toISOString(),
      data,
      emailSubject,
      emailBody,
      autoResponderSubject,
      autoResponderBody
    };

    try {
      const logsDir = path.join(process.cwd(), 'logs');
      await fs.mkdir(logsDir, { recursive: true });
      
      const logFile = path.join(logsDir, 'hire-submissions.jsonl');
      await fs.appendFile(logFile, JSON.stringify(submissionData) + '\n');
    } catch (fileError) {
      console.error('Failed to save submission to file:', fileError);
    }

    // Track analytics
    console.log('Hire form conversion:', {
      projectType: data.projectType,
      budget: data.budget,
      timeline: data.timeline,
      timestamp: new Date().toISOString()
    });

    return NextResponse.json({ 
      success: true, 
      message: 'Request submitted successfully' 
    });

  } catch (error) {
    console.error('Hire form submission error:', error);
    return NextResponse.json(
      { error: 'Internal server error' }, 
      { status: 500 }
    );
  }
}