import { NextRequest, NextResponse } from 'next/server';

interface HireFormData {
  description: string;
  projectType: string;
  budget: string;
  timeline: string;
  email: string;
}

const PROJECT_TYPES = new Set([
  'AI Agent',
  'Web App',
  'Automation',
  'Trading/Finance',
  'Other',
]);

const MAX_TELEGRAM_FIELD_LENGTH = 900;

function trimForNotification(value: string) {
  const normalized = value.trim();
  if (normalized.length <= MAX_TELEGRAM_FIELD_LENGTH) return normalized;
  return `${normalized.slice(0, MAX_TELEGRAM_FIELD_LENGTH - 1)}...`;
}

function buildTelegramMessage(data: HireFormData, submittedAt: string) {
  return [
    'New repo.box hire request',
    '',
    `Type: ${data.projectType}`,
    `Budget: ${data.budget || 'Not specified'}`,
    `Timeline: ${data.timeline}`,
    `Contact: ${data.email}`,
    '',
    trimForNotification(data.description),
    '',
    `Submitted: ${submittedAt}`,
  ].join('\n');
}

async function sendTelegramNotification(message: string) {
  const token = process.env.REPOBOX_HIRE_TELEGRAM_BOT_TOKEN;
  const chatId = process.env.REPOBOX_HIRE_TELEGRAM_CHAT_ID;
  const threadId = process.env.REPOBOX_HIRE_TELEGRAM_THREAD_ID;

  if (!token || !chatId) {
    console.warn('Hire form Telegram notification is not configured.');
    return { configured: false };
  }

  const response = await fetch(`https://api.telegram.org/bot${token}/sendMessage`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chat_id: chatId,
      message_thread_id: threadId ? Number(threadId) : undefined,
      text: message,
      disable_web_page_preview: true,
    }),
  });

  if (!response.ok) {
    const body = await response.text();
    throw new Error(`Telegram notification failed: ${response.status} ${body.slice(0, 300)}`);
  }

  return { configured: true };
}

export async function POST(request: NextRequest) {
  try {
    const data: HireFormData = await request.json();
    const submittedAt = new Date().toISOString();
    
    // Validate required fields
    if (!data.description || !data.projectType || !data.timeline || !data.email) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    if (!PROJECT_TYPES.has(data.projectType)) {
      return NextResponse.json({ error: 'Invalid project type' }, { status: 400 });
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

    // Format intake content for the local submissions log.
    const notificationSubject = `New Hire Request: ${data.projectType} Project`;
    const notificationBody = `
New hire request received from repo.box/hire

Project Details:
- Type: ${data.projectType}
- Budget: ${data.budget || 'Not specified'}
- Timeline: ${data.timeline}
- Contact: ${data.email}

Description:
${data.description}

---
Submitted: ${submittedAt}
Source: repo.box/hire
    `.trim();

    console.log('Hire form submission:', {
      subject: notificationSubject,
      body: notificationBody,
      timestamp: submittedAt
    });

    // Save submission to a local file for manual processing
    // This is a fallback until proper email integration is set up
    const fs = require('fs').promises;
    const path = require('path');
    
    const submissionData = {
      timestamp: submittedAt,
      data,
      notificationSubject,
      notificationBody
    };

    try {
      const logsDir = path.join(process.cwd(), 'logs');
      await fs.mkdir(logsDir, { recursive: true });
      
      const logFile = path.join(logsDir, 'hire-submissions.jsonl');
      await fs.appendFile(logFile, JSON.stringify(submissionData) + '\n');
    } catch (fileError) {
      console.error('Failed to save submission to file:', fileError);
    }

    let telegramStatus = { configured: false };
    try {
      telegramStatus = await sendTelegramNotification(buildTelegramMessage(data, submittedAt));
    } catch (notificationError) {
      console.error('Failed to send hire notification:', notificationError);
      return NextResponse.json(
        { error: 'Submission could not be delivered. Please try again.' },
        { status: 502 }
      );
    }

    // Track analytics
    console.log('Hire form conversion:', {
      projectType: data.projectType,
      budget: data.budget,
      timeline: data.timeline,
      notified: telegramStatus.configured,
      timestamp: submittedAt
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
