# File Viewer with Syntax Highlighting - Implementation Spec

**Project**: repo.box
**Component**: Explorer File Viewer  
**Priority**: P1
**Agent**: pm-agent
**Date**: 2026-03-21

## Overview

Enhance the current plain text file viewer in the repo explorer with proper syntax highlighting, line numbers, copy functionality, and enhanced metadata display. Transform the basic `<pre><code>` display into a rich, GitHub-style file viewer.

## Current State Analysis

**Location**: `web/src/app/explore/[address]/[name]/page.tsx`  
**Current Implementation**:
```tsx
<div className="explore-file-viewer">
  <div className="explore-file-header">
    <span className="explore-file-icon">{getFileIcon(fileContent.path, false)}</span>
    <span className="explore-file-name">{fileContent.path.split('/').pop()}</span>
  </div>
  <pre className="explore-file-content"><code>{fileContent.content}</code></pre>
</div>
```

**Libraries Available**: 
- `react-syntax-highlighter` v16.1.1 (already installed)
- `@types/react-syntax-highlighter` v15.5.13

## UX Wireframe/Mockup Description

### Enhanced File Header
```
[📄] filename.rs                    [Raw] [Copy] [Download]
├─ 1,234 lines • 45.2 KB • UTF-8
└─ Rust source file
```

### Enhanced File Content Area
```
┌─────────────────────────────────────────────────────────────┐
│  1  │ use std::collections::HashMap;                          │
│  2  │                                                         │
│  3  │ fn main() {                                            │
│  4  │     let mut map = HashMap::new();                      │
│  5  │     map.insert("key", "value");                        │
│ ... │ ...                                                     │
│ 100 │ }                                                       │
└─────────────────────────────────────────────────────────────┘
```

### Key UX Features
1. **Line Numbers**: Left gutter with padding, right-aligned numbers
2. **Syntax Highlighting**: Language-specific color coding
3. **Copy Button**: One-click copy entire file content to clipboard  
4. **Raw View Toggle**: Switch between formatted and raw plain text
5. **File Metadata**: Size, line count, encoding, file type description
6. **Download Option**: Direct file download link
7. **Responsive Design**: Mobile-friendly with horizontal scroll
8. **Keyboard Navigation**: Arrow keys, Page Up/Down support

## Component Structure (React/Next.js)

### New Component: `FileViewer`
**Location**: `web/src/components/explorer/FileViewer.tsx`

```tsx
interface FileViewerProps {
  filePath: string;
  fileContent: string;
  onClose?: () => void;
}

export function FileViewer({ filePath, fileContent, onClose }: FileViewerProps) {
  // Component implementation
}
```

### Sub-components Structure
1. **FileViewerHeader** - Metadata, buttons, file info
2. **FileViewerContent** - Syntax highlighted content area
3. **FileViewerToolbar** - Copy, raw toggle, download actions
4. **LineNumberGutter** - Left gutter with line numbers

### Integration Pattern
Replace the current `explore-file-viewer` section in the main repo page with:
```tsx
{fileContent ? (
  <FileViewer 
    filePath={fileContent.path}
    fileContent={fileContent.content}
    onClose={() => setFileContent(null)}
  />
) : (
  // existing file list
)}
```

## Libraries and Implementation Approach

### Primary: react-syntax-highlighter

**Choice Rationale**: 
- Already installed and typed
- Mature library with extensive language support
- Multiple theme options matching our dark design
- Tree-shakable with language-specific imports
- Server-side rendering compatible

**Implementation**:
```tsx
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/cjs/styles/prism';

// Language detection based on file extension
const getLanguageFromPath = (path: string): string => {
  const ext = path.split('.').pop()?.toLowerCase();
  const languageMap: Record<string, string> = {
    'rs': 'rust',
    'js': 'javascript', 
    'ts': 'typescript',
    'py': 'python',
    'go': 'go',
    'rb': 'ruby',
    'php': 'php',
    'java': 'java',
    'c': 'c',
    'cpp': 'cpp', 
    'h': 'c',
    'yml': 'yaml',
    'yaml': 'yaml',
    'json': 'json',
    'xml': 'xml',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sql': 'sql',
    'md': 'markdown',
    'sh': 'bash',
    'dockerfile': 'dockerfile'
  };
  return languageMap[ext || ''] || 'text';
};
```

### Theme Integration
- **Primary Theme**: `oneDark` (matches current dark design)
- **Fallback Theme**: `atomDark` 
- **Custom CSS Variables**: Map syntax highlighting colors to existing `--bp-*` variables
- **Line Number Styling**: Custom implementation using `showLineNumbers` prop

## File Size Limits and Performance Considerations

### File Size Strategy
1. **Small Files** (< 100KB): Full syntax highlighting
2. **Medium Files** (100KB - 1MB): Virtualized rendering with windowing
3. **Large Files** (> 1MB): Plain text fallback with warning
4. **Binary Files**: Hex viewer or "Binary file not shown" message

### Performance Optimizations
1. **Lazy Loading**: Import syntax highlighter components on demand
2. **Language Chunking**: Only load specific language definitions when needed
3. **Virtualization**: For files > 1000 lines, implement virtual scrolling
4. **Caching**: Memoize syntax highlighting results
5. **Progressive Enhancement**: Render plain text first, enhance with highlighting

### Implementation Approach
```tsx
const FileViewer = ({ filePath, fileContent }) => {
  const fileSize = new Blob([fileContent]).size;
  const lineCount = fileContent.split('\n').length;
  
  // Size-based rendering strategy
  if (fileSize > 1024 * 1024) {
    return <LargeFileViewer />; // Plain text + warning
  } else if (lineCount > 1000) {
    return <VirtualizedFileViewer />; // Windowed rendering
  } else {
    return <StandardFileViewer />; // Full highlighting
  }
};
```

## Supported File Types/Languages

### Tier 1 (Core Web/Systems Languages)
- **JavaScript/TypeScript**: `.js`, `.ts`, `.jsx`, `.tsx`, `.mjs`, `.cjs`
- **Rust**: `.rs`
- **Python**: `.py`, `.pyw` 
- **Go**: `.go`
- **C/C++**: `.c`, `.cpp`, `.cc`, `.cxx`, `.h`, `.hpp`
- **HTML/CSS**: `.html`, `.css`, `.scss`, `.sass`, `.less`

### Tier 2 (Common Languages)  
- **Java**: `.java`
- **PHP**: `.php`
- **Ruby**: `.rb`
- **Shell**: `.sh`, `.bash`, `.zsh`, `.fish`
- **SQL**: `.sql`
- **Dockerfile**: `Dockerfile`, `.dockerfile`

### Tier 3 (Configuration/Markup)
- **YAML**: `.yml`, `.yaml`
- **JSON**: `.json`, `.jsonc`
- **XML**: `.xml`
- **Markdown**: `.md`, `.mdx`
- **TOML**: `.toml`
- **INI**: `.ini`, `.cfg`, `.conf`

### Binary/Unsupported
- **Images**: Show image preview instead of hex
- **Archives**: `.zip`, `.tar`, `.gz` - Show file listing
- **Binary**: Show "Binary file not shown" message with file info

## Implementation Approach: Client-Side vs Server-Side

### Recommendation: Client-Side Rendering

**Rationale**:
1. **Better UX**: Instant highlighting without server round-trips
2. **Reduced Server Load**: Offload syntax processing to client
3. **Caching Benefits**: Browser can cache highlighted results
4. **Interactive Features**: Line selection, copy, etc. work instantly

**Server-Side Responsibilities**:
1. **File Content Delivery**: Raw content via existing blob API
2. **File Metadata**: Size, type, encoding detection
3. **Security**: Content sanitization and size limits

**Client-Side Responsibilities**:
1. **Syntax Highlighting**: react-syntax-highlighter processing
2. **Language Detection**: Extension-based mapping
3. **UI State**: Line selection, copy feedback, raw toggle
4. **Performance**: Virtualization, chunking large files

### API Contract (No Changes Required)
Current blob API endpoint works perfectly:
```
GET /api/explorer/repos/{address}/{name}/blob/{path}
Response: { path: string, content: string }
```

## Component Implementation Details

### FileViewer Component Structure
```tsx
// web/src/components/explorer/FileViewer.tsx
export function FileViewer({ filePath, fileContent, onClose }: FileViewerProps) {
  const [showRaw, setShowRaw] = useState(false);
  const [copied, setCopied] = useState(false);
  
  const language = getLanguageFromPath(filePath);
  const fileSize = new Blob([fileContent]).size;
  const lineCount = fileContent.split('\n').length;
  const fileName = filePath.split('/').pop() || '';
  
  return (
    <div className="file-viewer">
      <FileViewerHeader 
        fileName={fileName}
        filePath={filePath}
        fileSize={fileSize}
        lineCount={lineCount}
        language={language}
        onClose={onClose}
      />
      <FileViewerToolbar
        onCopy={() => copyToClipboard(fileContent)}
        onToggleRaw={() => setShowRaw(!showRaw)}
        onDownload={() => downloadFile(fileName, fileContent)}
        showRaw={showRaw}
        copied={copied}
      />
      <FileViewerContent
        content={fileContent}
        language={language}
        showRaw={showRaw}
        lineCount={lineCount}
      />
    </div>
  );
}
```

### CSS Classes and Styling
```css
/* Add to web/src/app/globals.css */

.file-viewer {
  background: var(--bp-surface);
  border: 1px solid var(--bp-border);
  border-radius: 12px;
  overflow: hidden;
  margin: 1rem 0;
}

.file-viewer-header {
  background: rgba(50, 100, 160, 0.1);
  padding: 12px 16px;
  border-bottom: 1px solid var(--bp-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.file-viewer-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--bp-heading);
}

.file-viewer-metadata {
  font-size: 11px;
  color: var(--bp-dim);
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-viewer-toolbar {
  background: rgba(50, 100, 160, 0.05);
  padding: 8px 16px;
  border-bottom: 1px solid var(--bp-border);
  display: flex;
  gap: 8px;
}

.file-viewer-btn {
  background: transparent;
  border: 1px solid var(--bp-border);
  color: var(--bp-text);
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s;
}

.file-viewer-btn:hover {
  background: rgba(79, 195, 247, 0.1);
  border-color: var(--bp-accent);
}

.file-viewer-btn.active {
  background: var(--bp-accent);
  color: var(--bp-bg);
}

.file-viewer-content {
  max-height: 600px;
  overflow: auto;
  font-family: var(--font-mono), monospace;
  font-size: 12px;
  line-height: 1.5;
}

/* Syntax highlighter overrides */
.file-viewer-content pre {
  margin: 0 !important;
  padding: 16px !important;
  background: transparent !important;
  border-radius: 0 !important;
  overflow-x: auto;
}

.file-viewer-content code {
  background: transparent !important;
  font-family: inherit !important;
  font-size: inherit !important;
}

/* Line numbers styling */
.file-viewer-line-numbers {
  color: var(--bp-dim) !important;
  background: rgba(50, 100, 160, 0.05) !important;
  border-right: 1px solid var(--bp-border) !important;
  padding-right: 12px !important;
  margin-right: 12px !important;
  user-select: none;
  min-width: 3em;
  text-align: right;
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  .file-viewer-header {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .file-viewer-toolbar {
    flex-wrap: wrap;
  }
  
  .file-viewer-content {
    max-height: 400px;
  }
}
```

## Testing Plan

### Unit Tests
**Framework**: Jest + React Testing Library  
**Location**: `web/src/components/explorer/__tests__/FileViewer.test.tsx`

**Test Cases**:
1. **Rendering Tests**
   - Renders file name and metadata correctly
   - Shows appropriate file icon
   - Displays line count and file size
   - Handles empty files gracefully

2. **Language Detection Tests**
   - Maps file extensions to correct languages
   - Falls back to 'text' for unknown extensions
   - Handles files without extensions
   - Supports multi-part extensions (`.d.ts`, `.spec.js`)

3. **Syntax Highlighting Tests**  
   - Applies syntax highlighting for supported languages
   - Shows plain text for unsupported files
   - Renders line numbers correctly
   - Handles very long lines without breaking layout

4. **Interactive Features Tests**
   - Copy button copies content to clipboard
   - Raw view toggle switches between highlighted and plain
   - Download button triggers file download
   - Close button calls onClose callback

5. **Performance Tests**
   - Large files (>1MB) show appropriate warning
   - Long files (>1000 lines) use virtualization
   - Binary files show "not shown" message
   - Loading states render correctly

### Integration Tests  
**Framework**: Playwright  
**Location**: `web/__tests__/e2e/file-viewer.test.ts`

**Test Scenarios**:
1. **Navigation to File**
   - Click file in tree → file viewer opens
   - Breadcrumb shows correct path
   - Back button returns to file list

2. **File Content Display**
   - Syntax highlighting applies correctly
   - Line numbers start from 1 and increment
   - File metadata shows accurate information
   - Scrolling works for long files

3. **User Interactions**
   - Copy button shows success feedback
   - Raw toggle changes display immediately  
   - Download starts file download
   - Keyboard navigation works (arrows, page up/down)

4. **Cross-Browser Testing**
   - Chrome: Modern syntax highlighting
   - Firefox: Line number alignment
   - Safari: Mobile responsive layout
   - Mobile: Touch scrolling and interaction

5. **Performance Testing**
   - Large file handling (1MB+)
   - Many file types in one repo
   - Rapid file switching
   - Memory usage with many open files

### Manual Testing Checklist
1. **Language Coverage**
   - [ ] Test each Tier 1 language
   - [ ] Verify Tier 2 fallbacks work
   - [ ] Check configuration file highlighting

2. **Edge Cases**
   - [ ] Empty files
   - [ ] Files with only whitespace  
   - [ ] Very long lines (>1000 chars)
   - [ ] Files with mixed line endings
   - [ ] Unicode/UTF-8 content
   - [ ] Binary files

3. **Mobile Experience**
   - [ ] Touch scrolling works smoothly
   - [ ] Buttons are appropriately sized
   - [ ] Horizontal overflow handled
   - [ ] Copy functionality works on mobile

4. **Accessibility**
   - [ ] Screen reader compatibility
   - [ ] Keyboard navigation
   - [ ] Color contrast compliance
   - [ ] Focus management

## Implementation Phases

### Phase 1: Core File Viewer (Week 1)
- [ ] Create `FileViewer` component with basic structure
- [ ] Implement syntax highlighting with react-syntax-highlighter
- [ ] Add language detection logic
- [ ] Style component to match design system
- [ ] Replace existing file viewer in repo page

### Phase 2: Enhanced Features (Week 2)  
- [ ] Add line numbers and copy functionality
- [ ] Implement raw view toggle
- [ ] Add file metadata display (size, line count)
- [ ] Implement download functionality
- [ ] Add responsive mobile styles

### Phase 3: Performance & Polish (Week 3)
- [ ] Add file size limits and performance optimizations
- [ ] Implement virtualization for large files
- [ ] Add loading states and error handling
- [ ] Comprehensive testing suite
- [ ] Cross-browser testing and fixes

### Phase 4: Advanced Features (Future)
- [ ] Line selection and highlighting
- [ ] Search within file functionality
- [ ] Split view for diffs
- [ ] Permalink to specific lines
- [ ] Blame/history integration

## Success Metrics

### User Experience
- **File View Time**: Reduce time from file click to readable content by 50%
- **User Feedback**: Positive feedback on syntax highlighting readability
- **Mobile Usage**: Maintain file viewing functionality on mobile devices

### Technical Performance
- **Load Time**: File viewer renders within 500ms for files <100KB
- **Memory Usage**: No memory leaks when switching between files
- **Browser Compatibility**: 100% functionality across modern browsers

### Coverage
- **Language Support**: 95% of files in typical repos have syntax highlighting
- **File Size Support**: Handle files up to 10MB without blocking UI
- **Error Handling**: Graceful fallbacks for unsupported formats

## Migration Strategy

### Deployment Approach
1. **Feature Flag**: Use environment variable to toggle new file viewer
2. **Gradual Rollout**: Enable for specific file types first
3. **Fallback**: Keep old viewer as backup during transition
4. **Monitoring**: Track error rates and performance impact

### CSS Migration
- Add new styles to `globals.css` without breaking existing styles
- Use specific class prefixes (`file-viewer-*`) to avoid conflicts
- Test with existing repo pages to ensure no regressions

### Component Integration
- Replace file viewer section in `page.tsx` with new component
- Maintain exact same props interface for seamless replacement
- Preserve existing state management and navigation

This implementation spec provides a comprehensive roadmap for transforming the basic file viewer into a powerful, GitHub-style code browser that enhances the repo.box explorer experience while maintaining performance and accessibility standards.