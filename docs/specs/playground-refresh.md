# Playground Refresh Technical Specification

**Priority**: P1  
**Tags**: playground, ui, ai  
**Date**: 2026-03-21  
**Author**: PM Agent  

## Executive Summary

This specification outlines the technical requirements for refreshing the repo.box playground with improved visual design, synchronized system prompts, faster model performance, and comprehensive testing coverage for both translation modes.

## Current State Assessment

### 1. Implementation Structure
- **Main playground page**: `/home/xiko/repobox/web/src/app/playground/page.tsx`
- **Client component**: `/home/xiko/repobox/web/src/components/playground/PlaygroundClient.tsx` 
- **System prompt**: `/home/xiko/repobox/web/src/lib/repobox-prompt.ts`
- **Styles**: `/home/xiko/repobox/web/src/app/globals.css` (lines 231-365)

### 2. Identified Issues

#### A. Missing Verbs in System Prompt (Critical)
Current prompt in `repobox-prompt.ts` only includes:
- **Branch verbs**: `push`, `merge`, `create`, `delete`, `force-push`
- **File verbs**: `edit`, `write`, `append`

**Missing from canonical source** (found in `/home/xiko/repobox/repobox-core/src/config.rs`):
- `read` (access verb)
- `branch` (branch creation verb)  
- `own` (special expansion verb that expands to all verbs)

#### B. Prompt Drift Risk
- System prompt is hardcoded in `repobox-prompt.ts`
- No automatic sync with canonical Rust implementation
- Risk of divergence as new verbs are added to core

#### C. Model Performance
- Currently using: `qwen3-235b-a22b-instruct-2507`
- Venice endpoint: `https://api.venice.ai/api/v1/chat/completions`
- Need faster model for better user experience

#### D. Visual Design Needs Refresh
- Current styles are functional but could be more modern
- Limited visual feedback during streaming
- Basic syntax highlighting implementation

## Technical Requirements

### 1. System Prompt Synchronization

#### 1.1 Create Canonical Prompt Source
**File**: `/home/xiko/repobox/repobox-core/src/prompt.rs`

```rust
// New file to be created
pub const REPOBOX_SYSTEM_PROMPT: &str = r#"
You generate .repobox/config.yml files. This is a PROPRIETARY format for repo.box, a git permission layer for AI agents. Do NOT invent fields or use any other YAML schema.

// ... (full prompt with ALL verbs including read, branch, own)
"#;
```

#### 1.2 Build-time Prompt Generation
**File**: `/home/xiko/repobox/scripts/generate-prompt.rs`

```rust
// Script to extract prompt from Rust code and generate TypeScript
use repobox_core::prompt::REPOBOX_SYSTEM_PROMPT;

fn main() {
    let ts_content = format!(
        "// GENERATED FILE - DO NOT EDIT MANUALLY\n// Generated from repobox-core/src/prompt.rs\n\nexport const REPOBOX_SYSTEM_PROMPT = `{}`;\n",
        REPOBOX_SYSTEM_PROMPT
    );
    
    std::fs::write("web/src/lib/generated-prompt.ts", ts_content)
        .expect("Failed to write generated prompt");
}
```

#### 1.3 Updated TypeScript Import
**File**: `/home/xiko/repobox/web/src/lib/repobox-prompt.ts`

```typescript
// Remove hardcoded prompt, import from generated file
export { REPOBOX_SYSTEM_PROMPT } from './generated-prompt';

// Keep Venice config here
export const VENICE_ENDPOINT = "https://api.venice.ai/api/v1/chat/completions";
export const VENICE_MODEL = "claude-sonnet-3.5-next"; // Updated model
```

### 2. Model Performance Optimization

#### 2.1 Model Selection
**Current**: `qwen3-235b-a22b-instruct-2507` (slow, high latency)  
**Recommended**: `claude-sonnet-3.5-next` (faster, better for structured output)

**Alternative options** (to evaluate):
- `gpt-4o-mini` (fastest, good for simple tasks)
- `claude-haiku-3.5` (balanced speed/quality)

#### 2.2 Request Optimization
**File**: `/home/xiko/repobox/web/src/components/playground/PlaygroundClient.tsx`

```typescript
// Optimized request parameters
const requestBody = {
  model: VENICE_MODEL,
  stream: true,
  messages: [
    { role: "system", content: REPOBOX_SYSTEM_PROMPT },
    { role: "user", content: userMessage }
  ],
  temperature: 0.1, // Reduced for more consistent output
  max_tokens: 1500,  // Reduced from 2000
  // Model-specific optimizations
  ...(VENICE_MODEL.includes('claude') ? {
    anthropic_parameters: {
      include_anthropic_system_prompt: false,
    }
  } : {
    venice_parameters: {
      include_venice_system_prompt: false,
      disable_thinking: true,
    }
  })
};
```

### 3. Visual Design Refresh

#### 3.1 Enhanced Color Palette
**File**: `/home/xiko/repobox/web/src/app/globals.css`

```css
:root {
  /* Enhanced playground-specific colors */
  --playground-primary: #4fc3f7;
  --playground-secondary: #81d4fa;
  --playground-accent: #00bcd4;
  --playground-success: #4caf50;
  --playground-warning: #ff9800;
  --playground-error: #f44336;
  --playground-surface-1: #0a1628;
  --playground-surface-2: #0d1f35;
  --playground-surface-3: #1a2d42;
}
```

#### 3.2 Modern Component Styles
```css
/* Enhanced mode toggle */
.playground-mode-btn {
  padding: 12px 24px;
  border: 2px solid var(--bp-border);
  border-radius: 8px;
  background: var(--playground-surface-2);
  color: var(--bp-text);
  font-weight: 500;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.playground-mode-btn.active {
  background: linear-gradient(135deg, var(--playground-primary), var(--playground-secondary));
  border-color: var(--playground-primary);
  color: white;
  box-shadow: 0 4px 16px rgba(79, 195, 247, 0.3);
}

/* Enhanced textarea */
.playground-textarea {
  border: 2px solid var(--bp-border);
  border-radius: 12px;
  background: var(--playground-surface-1);
  transition: all 0.3s ease;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
}

.playground-textarea:focus {
  border-color: var(--playground-primary);
  box-shadow: 0 0 0 3px rgba(79, 195, 247, 0.1);
}

/* Enhanced output */
.playground-output {
  border-radius: 12px;
  background: var(--playground-surface-1);
  border: 2px solid var(--bp-border);
  position: relative;
  overflow: hidden;
}

.playground-output.streaming {
  border-color: var(--playground-primary);
}

.playground-output.streaming::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--playground-primary), transparent);
  animation: shimmer 2s infinite;
}

@keyframes shimmer {
  0% { left: -100%; }
  100% { left: 100%; }
}
```

#### 3.3 Enhanced Syntax Highlighting
**File**: `/home/xiko/repobox/web/src/components/playground/SyntaxHighlighter.tsx`

```typescript
// New component for better highlighting
export function YamlHighlighter({ content }: { content: string }) {
  return (
    <div className="syntax-highlighter">
      {/* Enhanced YAML syntax highlighting with better color scheme */}
      <pre dangerouslySetInnerHTML={{ __html: highlightYamlAdvanced(content) }} />
    </div>
  );
}

function highlightYamlAdvanced(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    // Comments
    .replace(/(#.*$)/gm, '<span class="yaml-comment">$1</span>')
    // Keys
    .replace(/^(\s*)([\w-]+)(:)/gm, '$1<span class="yaml-key">$2</span><span class="yaml-colon">$3</span>')
    // Strings
    .replace(/(&quot;[^&]*&quot;|'[^']*')/g, '<span class="yaml-string">$1</span>')
    // EVM addresses
    .replace(/(evm:0x[\w.]+)/g, '<span class="yaml-address">$1</span>')
    // Branch refs
    .replace(/(&gt;[\w*\/\-\.]+)/g, '<span class="yaml-branch">$1</span>')
    // File paths
    .replace(/(\.\/[\w.*\/-]+)/g, '<span class="yaml-path">$1</span>')
    // List markers
    .replace(/^(\s*)(- )/gm, '$1<span class="yaml-marker">$2</span>');
}
```

### 4. Comprehensive Testing Implementation

#### 4.1 Test Mode Infrastructure
**File**: `/home/xiko/repobox/web/src/components/playground/TestRunner.tsx`

```typescript
interface TestCase {
  id: string;
  name: string;
  mode: 'generate' | 'explain';
  input: string;
  expectedPatterns: string[];
  shouldNotContain?: string[];
}

export const TEST_CASES: TestCase[] = [
  // English → Config tests
  {
    id: 'gen-basic-team',
    name: 'Basic team with agents',
    mode: 'generate',
    input: 'Two founders with full access. One AI agent that can only push to feature branches.',
    expectedPatterns: [
      'groups:',
      'founders:',
      '- evm:0x',
      'permissions:',
      'push >feature/**'
    ]
  },
  // Config → English tests  
  {
    id: 'exp-complex-config',
    name: 'Complex permissions explanation',
    mode: 'explain',
    input: `groups:\n  maintainers:\n    - evm:0xAAA...111\npermissions:\n  default: deny\n  rules:\n    - maintainers own *`,
    expectedPatterns: [
      'maintainers can',
      'full access',
      'default deny'
    ]
  }
  // ... more test cases
];
```

#### 4.2 Test Execution
**File**: `/home/xiko/repobox/web/src/components/playground/PlaygroundClient.tsx`

```typescript
// Add test mode to existing component
const [testMode, setTestMode] = useState(false);
const [testResults, setTestResults] = useState<TestResult[]>([]);

const runTests = async () => {
  setTestMode(true);
  const results: TestResult[] = [];
  
  for (const testCase of TEST_CASES) {
    const result = await executeTest(testCase);
    results.push(result);
    setTestResults([...results]);
  }
  
  setTestMode(false);
};

const executeTest = async (testCase: TestCase): Promise<TestResult> => {
  // Run the same logic as normal playground but validate output
  // against expected patterns
};
```

## Implementation Plan

### Phase 1: System Prompt Synchronization (2 days)
1. **Day 1**: Create canonical prompt source in Rust
   - [ ] Add `/home/xiko/repobox/repobox-core/src/prompt.rs` with complete verb list
   - [ ] Update verb documentation to include `read`, `branch`, `own`
   - [ ] Add `own` expansion logic documentation

2. **Day 2**: Implement build-time generation
   - [ ] Create `scripts/generate-prompt.rs`
   - [ ] Update web build process to run generation
   - [ ] Test prompt synchronization

### Phase 2: Model Performance Optimization (1 day)
3. **Day 3**: Model and request optimization
   - [ ] Evaluate and select faster Venice model
   - [ ] Implement optimized request parameters
   - [ ] Add model-specific parameter handling
   - [ ] Performance testing and validation

### Phase 3: Visual Design Refresh (2 days)
4. **Day 4**: Enhanced styling and components
   - [ ] Implement new color palette
   - [ ] Update component styles with animations
   - [ ] Create enhanced syntax highlighter
   - [ ] Add loading states and micro-interactions

5. **Day 5**: Responsive improvements and polish
   - [ ] Mobile-responsive design updates
   - [ ] Accessibility improvements
   - [ ] Error state designs
   - [ ] Cross-browser testing

### Phase 4: Comprehensive Testing (2 days)
6. **Day 6**: Test infrastructure
   - [ ] Create test case definitions
   - [ ] Implement test runner component
   - [ ] Add test result visualization
   - [ ] Create test mode UI

7. **Day 7**: Test coverage and validation
   - [ ] Develop comprehensive test suite
   - [ ] Implement both modes testing (English↔Config)
   - [ ] Add edge case coverage
   - [ ] Performance benchmarking

## File Changes Required

### New Files
1. `/home/xiko/repobox/repobox-core/src/prompt.rs` - Canonical system prompt
2. `/home/xiko/repobox/scripts/generate-prompt.rs` - Build-time generator
3. `/home/xiko/repobox/web/src/lib/generated-prompt.ts` - Generated TypeScript prompt
4. `/home/xiko/repobox/web/src/components/playground/TestRunner.tsx` - Test infrastructure
5. `/home/xiko/repobox/web/src/components/playground/SyntaxHighlighter.tsx` - Enhanced highlighting

### Modified Files
1. `/home/xiko/repobox/web/src/lib/repobox-prompt.ts` - Remove hardcoded prompt, add model config
2. `/home/xiko/repobox/web/src/components/playground/PlaygroundClient.tsx` - Add testing, optimize requests
3. `/home/xiko/repobox/web/src/app/globals.css` - Enhanced playground styles
4. `/home/xiko/repobox/web/package.json` - Build script updates
5. `/home/xiko/repobox/Cargo.toml` - Add prompt generation workspace member

## Testing Strategy

### Unit Testing
- System prompt parsing validation
- Syntax highlighter output verification
- Model request/response handling

### Integration Testing
- Full English→Config generation flow
- Full Config→English explanation flow
- Error handling and edge cases

### Performance Testing
- Model response time measurement
- UI responsiveness during streaming
- Large config handling

### Manual Testing Checklist
- [ ] All example inputs generate valid YAML
- [ ] All example configs produce accurate explanations  
- [ ] Visual design consistency across browsers
- [ ] Mobile responsiveness
- [ ] Accessibility compliance
- [ ] Error states display correctly

## Success Criteria

1. **Prompt Accuracy**: System prompt includes all verbs from canonical Rust source
2. **Performance**: Model responses 2x faster than current implementation
3. **Visual Quality**: Modern, polished interface with smooth animations
4. **Test Coverage**: 95%+ test case success rate for both translation modes
5. **Maintainability**: Automatic prompt synchronization prevents future drift

## Risk Mitigation

### Technical Risks
- **Model availability**: Have fallback models configured
- **Build complexity**: Keep generation script simple and well-documented
- **Performance regression**: Benchmark before/after implementation

### UX Risks  
- **Learning curve**: Provide clear visual feedback during changes
- **Functionality loss**: Ensure backward compatibility with existing examples
- **Error handling**: Comprehensive error states for all failure modes

## Future Considerations

- **Model fine-tuning**: Custom model training on repo.box config format
- **Advanced testing**: Property-based testing for config generation
- **Real-time collaboration**: Multi-user playground sessions
- **Version control**: Save and share playground configurations

---

**Estimated effort**: 7 developer days  
**Dependencies**: Venice API access, Rust build toolchain  
**Reviewers**: Development team, UX team  
**Success metrics**: Response time, test pass rate, user satisfaction scores