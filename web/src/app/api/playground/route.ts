import { NextRequest, NextResponse } from "next/server";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { REPOBOX_SYSTEM_PROMPT, VENICE_ENDPOINT, VENICE_MODEL } from "@/lib/repobox-prompt";

const execFileAsync = promisify(execFile);

type Mode = "generate" | "explain";

interface LintResult {
  ok: boolean;
  output: string;
}

interface VeniceResponse {
  choices?: Array<{ message?: { content?: string } }>;
}

function deterministicGuards(yaml: string): string[] {
  const issues: string[] = [];

  // Common malformed quoting / target corruption patterns seen in playground failures
  if (/branch"\s*>>\*/i.test(yaml) || /\">>\*/.test(yaml)) {
    issues.push('Malformed branch target quoting (e.g. branch">>*).');
  }
  if (/“|”|‘|’/.test(yaml)) {
    issues.push("Smart quotes detected — use plain ASCII quotes only.");
  }
  if (/\bnot\s+push\s*:\s*/i.test(yaml)) {
    issues.push('Invalid nested "not push:" mapping style. Use flat rule: "group not push >main".');
  }

  return issues;
}

async function runLint(yaml: string): Promise<LintResult> {
  const dir = await mkdtemp(join(tmpdir(), "repobox-pg-"));
  try {
    const cfgDir = join(dir, ".repobox");
    const cfgPath = join(cfgDir, "config.yml");
    await mkdir(cfgDir, { recursive: true });
    await writeFile(cfgPath, yaml, "utf8");

    try {
      const { stdout, stderr } = await execFileAsync("git", ["repobox", "lint"], {
        cwd: dir,
        timeout: 15000,
      });
      return { ok: true, output: `${stdout}${stderr}`.trim() };
    } catch (err: any) {
      const out = `${err?.stdout || ""}${err?.stderr || ""}`.trim();
      return { ok: false, output: out || String(err?.message || err) };
    }
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

function normalizeOutput(raw: string): string {
  return raw.replace(/^```[a-zA-Z]*\n?/gm, "").replace(/```/g, "").trim();
}

async function callVenice(userMessage: string): Promise<string> {
  const apiKey = process.env.VENICE_API_KEY || process.env.NEXT_PUBLIC_VENICE_API_KEY;
  if (!apiKey) throw new Error("Missing Venice API key.");

  const res = await fetch(VENICE_ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify({
      model: VENICE_MODEL,
      stream: false,
      messages: [
        {
          role: "system",
          content: REPOBOX_SYSTEM_PROMPT,
        },
        {
          role: "user",
          content: userMessage,
        },
      ],
      temperature: 0.05,
      max_tokens: 1500,
      ...(VENICE_MODEL.includes("claude")
        ? {
            anthropic_parameters: {
              include_anthropic_system_prompt: false,
            },
          }
        : {
            venice_parameters: {
              include_venice_system_prompt: false,
              disable_thinking: true,
            },
          }),
    }),
  });

  if (!res.ok) {
    throw new Error(`Venice API error: ${res.status}`);
  }

  const data = (await res.json()) as VeniceResponse;
  return normalizeOutput(data.choices?.[0]?.message?.content || "");
}

function strictGeneratePrompt(input: string): string {
  return `Generate a .repobox/config.yml for this scenario:\n\n${input}\n\nHARD CONSTRAINTS:\n- Output ONLY raw YAML (no prose, no markdown fences).\n- Use ONLY Format A flat string rules under permissions.rules.\n- Every address must be valid 40-hex EVM format (e.g. evm:0x1111111111111111111111111111111111111111).\n- Do not use placeholder ellipsis addresses like 0xAAA...111.\n- Ensure output passes git repobox lint.`;
}

function repairPrompt(previousYaml: string, lintOutput: string, guardIssues: string[]): string {
  const guards = guardIssues.length > 0 ? `\nDeterministic guard failures:\n- ${guardIssues.join("\n- ")}` : "";
  return `Fix this .repobox/config.yml so it passes git repobox lint.\n\nCurrent YAML:\n${previousYaml}\n\nLint output:\n${lintOutput}${guards}\n\nRules:\n- Return ONLY corrected YAML.\n- Keep intent as close as possible.\n- Use valid 40-hex evm addresses (no ellipsis placeholders).\n- Use flat string rule format only.`;
}

export async function POST(req: NextRequest) {
  try {
    const body = (await req.json()) as { mode?: Mode; input?: string };
    const mode = body.mode;
    const input = (body.input || "").trim();

    if (!mode || !["generate", "explain"].includes(mode)) {
      return NextResponse.json({ error: "Invalid mode" }, { status: 400 });
    }
    if (!input) {
      return NextResponse.json({ error: "Input required" }, { status: 400 });
    }

    if (mode === "explain") {
      const prompt = `Explain this .repobox/config.yml in plain English. What can each group do? What are they denied?\n\n${input}`;
      const output = await callVenice(prompt);
      return NextResponse.json({ output, lint: null });
    }

    // Generate mode with lint gate + one repair pass
    const generated = await callVenice(strictGeneratePrompt(input));
    const guardIssues = deterministicGuards(generated);
    let lint = await runLint(generated);

    if ((!lint.ok || guardIssues.length > 0) && generated) {
      const repaired = await callVenice(repairPrompt(generated, lint.output, guardIssues));
      const repairedGuards = deterministicGuards(repaired);
      const repairedLint = await runLint(repaired);
      if (repairedLint.ok && repairedGuards.length === 0) {
        return NextResponse.json({
          output: repaired,
          lint: { ok: true, output: repairedLint.output, repaired: true, guardIssues: [] },
        });
      }

      return NextResponse.json({
        output: repaired,
        lint: {
          ok: false,
          output: repairedLint.output,
          repaired: true,
          guardIssues: repairedGuards,
        },
      });
    }

    return NextResponse.json({
      output: generated,
      lint: { ok: true, output: lint.output, repaired: false, guardIssues: [] },
    });
  } catch (error: any) {
    return NextResponse.json({ error: error?.message || "Request failed" }, { status: 500 });
  }
}
