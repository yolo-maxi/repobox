/**
 * Enhanced syntax highlighter for YAML configs and explanations
 */

export function YamlHighlighter({ content }: { content: string }) {
  return (
    <div className="syntax-highlighter">
      <pre dangerouslySetInnerHTML={{ __html: highlightYamlAdvanced(content) }} />
    </div>
  );
}

export function ExplanationHighlighter({ content }: { content: string }) {
  return (
    <div className="explanation-highlighter">
      <div dangerouslySetInnerHTML={{ __html: highlightExplanationAdvanced(content) }} />
    </div>
  );
}

function highlightYamlAdvanced(text: string): string {
  // Remove markdown code fences first
  text = text.replace(/^```ya?ml\n?/gm, "").replace(/^```\n?/gm, "");

  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    // Comments
    .replace(/(#.*$)/gm, '<span class="yaml-comment">$1</span>')
    // Keys (group names, permission properties)
    .replace(/^(\s*)([\w-]+)(:)/gm, '$1<span class="yaml-key">$2</span><span class="yaml-colon">$3</span>')
    // Strings in quotes
    .replace(/(&quot;[^&]*&quot;|'[^']*')/g, '<span class="yaml-string">$1</span>')
    // EVM addresses
    .replace(/(evm:0x[\w.]+)/g, '<span class="yaml-address">$1</span>')
    // Branch refs (>main, >feature/**)
    .replace(/(&gt;[\w*\/\-\.]+)/g, '<span class="yaml-branch">$1</span>')
    // File paths (./contracts/**, ./.repobox/config.yml)
    .replace(/(\.\/[\w.*\/-]+)/g, '<span class="yaml-path">$1</span>')
    // List markers
    .replace(/^(\s*)(- )/gm, '$1<span class="yaml-marker">$2</span>')
    // Verbs (push, merge, own, etc.)
    .replace(/\b(push|merge|branch|create|delete|force-push|edit|write|append|read|own|not)\b/g, '<span class="yaml-verb">$1</span>')
    // Default policy values
    .replace(/\b(allow|deny)\b/g, '<span class="yaml-policy">$1</span>');
}

function highlightExplanationAdvanced(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    // Bold text
    .replace(/\*\*([^*]+)\*\*/g, '<span class="exp-bold">$1</span>')
    // Inline code
    .replace(/`([^`]+)`/g, '<span class="exp-code">$1</span>')
    // List markers
    .replace(/^(\s*[-•])/gm, '<span class="exp-marker">$1</span>')
    // Icons/emojis
    .replace(/✅/g, '<span class="exp-allowed">✅</span>')
    .replace(/❌/g, '<span class="exp-denied">❌</span>')
    // Permissions keywords
    .replace(/\b(can|cannot|allowed|denied|permitted|forbidden|read|write|push|merge|branch|create|delete|edit|append|force-push)\b/gi, '<span class="exp-permission">$1</span>')
    // Branch names
    .replace(/\b(main|master|dev|feature|fix|contributor)\b/g, '<span class="exp-branch">$1</span>')
    // Group names in explanations
    .replace(/\b(founders?|maintainers?|agents?|viewers?|workers?|contributors?|orchestrator)\b/g, '<span class="exp-group">$1</span>');
}