'use client'
import { useState, useEffect, useCallback } from 'react'
import ReactMarkdown from 'react-markdown'

type Test = { text: string; status: 'pending' | 'passing' | 'failing' }
type Component = { name: string; tests: Test[] }
type Spec = { name: string; filename: string; content: string }
type Story = { name: string; steps: string[]; status: 'draft' | 'tested' | 'verified'; notes: string }

export default function Dashboard() {
  const [tab, setTab] = useState('overview')
  const [components, setComponents] = useState<Component[]>([])
  const [specs, setSpecs] = useState<Spec[]>([])
  const [openComponents, setOpenComponents] = useState<Set<number>>(new Set())
  const [editingSpec, setEditingSpec] = useState<string | null>(null)
  const [specDraft, setSpecDraft] = useState('')
  const [editingTest, setEditingTest] = useState<string | null>(null)
  const [newTestText, setNewTestText] = useState('')
  const [saving, setSaving] = useState(false)
  const [stories, setStories] = useState<Story[]>([])
  const [openStories, setOpenStories] = useState<Set<number>>(new Set())
  const [editingStory, setEditingStory] = useState<number | null>(null)
  const [storyDraft, setStoryDraft] = useState<Story | null>(null)
  const [skillContent, setSkillContent] = useState('')
  const [editingSkill, setEditingSkill] = useState(false)
  const [skillDraft, setSkillDraft] = useState('')

  useEffect(() => {
    fetch('/api/tests').then(r => r.json()).then(setComponents)
    fetch('/api/specs').then(r => r.json()).then(setSpecs)
    fetch('/api/stories').then(r => r.json()).then(setStories)
    fetch('/api/skill').then(r => r.json()).then(d => setSkillContent(d.content))
  }, [])

  const saveTests = useCallback(async (updated: Component[]) => {
    setComponents(updated)
    await fetch('/api/tests', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(updated) })
  }, [])

  const toggleStatus = (ci: number, ti: number) => {
    const updated = [...components]
    const test = updated[ci].tests[ti]
    const cycle: Record<string, string> = { pending: 'passing', passing: 'failing', failing: 'pending' }
    test.status = cycle[test.status] as Test['status']
    saveTests(updated)
  }

  const addTest = (ci: number) => {
    if (!newTestText.trim()) return
    const updated = [...components]
    updated[ci].tests.push({ text: newTestText.trim(), status: 'pending' })
    saveTests(updated)
    setNewTestText('')
    setEditingTest(null)
  }

  const removeTest = (ci: number, ti: number) => {
    const updated = [...components]
    updated[ci].tests.splice(ti, 1)
    saveTests(updated)
  }

  const editTestText = (ci: number, ti: number, text: string) => {
    const updated = [...components]
    updated[ci].tests[ti].text = text
    saveTests(updated)
  }

  const addComponent = () => {
    const name = prompt('Component name:')
    if (!name) return
    const updated = [...components, { name, tests: [] }]
    saveTests(updated)
  }

  const saveSpec = async (name: string) => {
    setSaving(true)
    await fetch(`/api/specs/${name}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: specDraft }) })
    const updated = specs.map(s => s.name === name ? { ...s, content: specDraft } : s)
    setSpecs(updated)
    setEditingSpec(null)
    setSaving(false)
  }

  const totalTests = components.reduce((a, c) => a + c.tests.length, 0)
  const passingTests = components.reduce((a, c) => a + c.tests.filter(t => t.status === 'passing').length, 0)
  const failingTests = components.reduce((a, c) => a + c.tests.filter(t => t.status === 'failing').length, 0)

  const toggleOpen = (i: number) => {
    const s = new Set(openComponents)
    s.has(i) ? s.delete(i) : s.add(i)
    setOpenComponents(s)
  }

  return (
    <div style={styles.container}>
      <h1 style={styles.h1}>🪸 repo.box</h1>
      <p style={styles.subtitle}>Build Dashboard — Synthesis Hackathon (deadline: Mar 23)</p>

      <div style={styles.tabs}>
        {['overview', 'tests', 'stories', 'specs', 'skill'].map(t => (
          <button key={t} onClick={() => setTab(t)} style={{ ...styles.tab, ...(tab === t ? styles.tabActive : {}) }}>
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      {/* STATS */}
      <div style={styles.stats}>
        <div style={styles.stat}><div style={{ ...styles.statValue, color: '#60a5fa' }}>{totalTests}</div><div style={styles.statLabel}>Total</div></div>
        <div style={styles.stat}><div style={{ ...styles.statValue, color: '#4ade80' }}>{passingTests}</div><div style={styles.statLabel}>Passing</div></div>
        <div style={styles.stat}><div style={{ ...styles.statValue, color: '#f87171' }}>{failingTests}</div><div style={styles.statLabel}>Failing</div></div>
        <div style={styles.stat}><div style={{ ...styles.statValue, color: '#facc15' }}>{totalTests - passingTests - failingTests}</div><div style={styles.statLabel}>Pending</div></div>
      </div>

      {/* OVERVIEW */}
      {tab === 'overview' && (
        <div>
          <div style={styles.pitch}>
            <strong style={{ color: '#fff' }}>repo.box</strong> makes git repositories safe for AI agents. 
            It shims the <code style={styles.code}>git</code> command so agents use normal git workflows — but every commit, merge, and push is silently checked against a <code style={styles.code}>.repobox.yml</code> file before it lands. 
            Each agent gets its own EVM keypair as identity. Each commit is signed. Permissions live in the repo, not on a server. Five agents, five keys, one repo, zero risk.
          </div>
          <h3 style={styles.sectionTitle}>TIMELINE</h3>
          {[
            { date: 'Mar 17 ✅', title: 'Design & Spec', desc: 'Four primitives designed. Five specs written. Test plan created.' },
            { date: 'Mar 18-19 🔨', title: 'Foundation', desc: '.repobox.yml parser. Permission engine. Git shim skeleton. Identity.' },
            { date: 'Mar 20-21', title: 'Enforcement', desc: 'git commit/merge/push interception. Branch create/delete. Edge cases.' },
            { date: 'Mar 22-23', title: 'Demo & Ship', desc: 'Landing page. Demo video. README. Hackathon submission.' },
          ].map((item, i) => (
            <div key={i} style={styles.timelineItem}>
              <div style={{ ...styles.timelineDot, background: i === 0 ? '#4ade80' : i === 1 ? '#facc15' : '#333' }} />
              <div style={styles.timelineDate}>{item.date}</div>
              <div style={styles.timelineTitle}>{item.title}</div>
              <div style={styles.timelineDesc}>{item.desc}</div>
            </div>
          ))}
        </div>
      )}

      {/* TESTS */}
      {tab === 'tests' && (
        <div>
          {components.map((comp, ci) => {
            const passed = comp.tests.filter(t => t.status === 'passing').length
            const failed = comp.tests.filter(t => t.status === 'failing').length
            const isOpen = openComponents.has(ci)
            return (
              <div key={ci} style={styles.component}>
                <div style={styles.componentHeader} onClick={() => toggleOpen(ci)}>
                  <span style={styles.componentName}>{comp.name}</span>
                  <span style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                    {failed > 0 && <span style={{ ...styles.badge, background: '#3b1111', color: '#f87171', borderColor: '#7f1d1d' }}>{failed} ✗</span>}
                    <span style={{
                      ...styles.badge,
                      ...(passed === comp.tests.length && comp.tests.length > 0 ? { background: '#052e16', color: '#4ade80', borderColor: '#166534' } : {})
                    }}>{passed}/{comp.tests.length}</span>
                    <span style={{ color: '#555', transition: 'transform 0.2s', transform: isOpen ? 'rotate(90deg)' : 'none' }}>›</span>
                  </span>
                </div>
                {isOpen && (
                  <div style={styles.componentBody}>
                    {comp.tests.map((test, ti) => (
                      <div key={ti} style={styles.test}>
                        <button
                          onClick={() => toggleStatus(ci, ti)}
                          style={{
                            ...styles.statusBtn,
                            background: test.status === 'passing' ? '#052e16' : test.status === 'failing' ? '#3b1111' : '#1a1a1a',
                            color: test.status === 'passing' ? '#4ade80' : test.status === 'failing' ? '#f87171' : '#666',
                            borderColor: test.status === 'passing' ? '#166534' : test.status === 'failing' ? '#7f1d1d' : '#333',
                          }}
                        >
                          {test.status === 'passing' ? '✓' : test.status === 'failing' ? '✗' : '○'}
                        </button>
                        <span style={{
                          flex: 1, fontSize: '0.85rem', lineHeight: 1.4,
                          color: test.status === 'passing' ? '#666' : '#e0e0e0',
                          textDecoration: test.status === 'passing' ? 'line-through' : 'none',
                        }}
                          contentEditable
                          suppressContentEditableWarning
                          onBlur={(e) => editTestText(ci, ti, e.currentTarget.textContent || '')}
                        >
                          {test.text}
                        </span>
                        <button onClick={() => removeTest(ci, ti)} style={styles.removeBtn}>×</button>
                      </div>
                    ))}
                    {editingTest === `${ci}` ? (
                      <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                        <input
                          autoFocus
                          style={styles.input}
                          value={newTestText}
                          onChange={e => setNewTestText(e.target.value)}
                          onKeyDown={e => e.key === 'Enter' && addTest(ci)}
                          placeholder="Test description..."
                        />
                        <button onClick={() => addTest(ci)} style={styles.addBtn}>Add</button>
                        <button onClick={() => setEditingTest(null)} style={styles.cancelBtn}>✕</button>
                      </div>
                    ) : (
                      <button onClick={() => setEditingTest(`${ci}`)} style={styles.addTestBtn}>+ Add test</button>
                    )}
                  </div>
                )}
              </div>
            )
          })}
          <button onClick={addComponent} style={styles.addComponentBtn}>+ Add component</button>
        </div>
      )}

      {/* SPECS */}
      {/* STORIES */}
      {tab === 'stories' && (
        <div>
          {stories.map((story, si) => {
            const isOpen = openStories.has(si)
            const isEditing = editingStory === si
            return (
              <div key={si} style={styles.component}>
                <div style={styles.componentHeader} onClick={() => {
                  const s = new Set(openStories); s.has(si) ? s.delete(si) : s.add(si); setOpenStories(s)
                }}>
                  <span style={styles.componentName}>{story.name}</span>
                  <span style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                    <span style={{
                      ...styles.badge,
                      ...(story.status === 'verified' ? { background: '#052e16', color: '#4ade80', borderColor: '#166534' } :
                         story.status === 'tested' ? { background: '#1a1a2e', color: '#60a5fa', borderColor: '#1e3a5f' } : {})
                    }}>{story.status}</span>
                    <span style={{ color: '#555', transition: 'transform 0.2s', transform: isOpen ? 'rotate(90deg)' : 'none' }}>›</span>
                  </span>
                </div>
                {isOpen && (
                  <div style={styles.componentBody}>
                    {isEditing && storyDraft ? (
                      <>
                        <input style={{ ...styles.input, marginBottom: 8, fontWeight: 600 }} value={storyDraft.name}
                          onChange={e => setStoryDraft({ ...storyDraft, name: e.target.value })} />
                        {storyDraft.steps.map((step, i) => (
                          <div key={i} style={{ display: 'flex', gap: 8, marginBottom: 4 }}>
                            <span style={{ color: '#555', fontSize: '0.8rem', minWidth: 20 }}>{i + 1}.</span>
                            <input style={{ ...styles.input, flex: 1 }} value={step}
                              onChange={e => { const s = [...storyDraft.steps]; s[i] = e.target.value; setStoryDraft({ ...storyDraft, steps: s }) }} />
                            <button onClick={() => { const s = [...storyDraft.steps]; s.splice(i, 1); setStoryDraft({ ...storyDraft, steps: s }) }}
                              style={styles.removeBtn}>×</button>
                          </div>
                        ))}
                        <button onClick={() => setStoryDraft({ ...storyDraft, steps: [...storyDraft.steps, ''] })}
                          style={styles.addTestBtn}>+ Add step</button>
                        <textarea style={{ ...styles.input, marginTop: 8, minHeight: 60, resize: 'vertical' as const }}
                          placeholder="Notes / open questions..."
                          value={storyDraft.notes} onChange={e => setStoryDraft({ ...storyDraft, notes: e.target.value })} />
                        <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                          <select style={{ ...styles.input, flex: 0 }} value={storyDraft.status}
                            onChange={e => setStoryDraft({ ...storyDraft, status: e.target.value as Story['status'] })}>
                            <option value="draft">Draft</option><option value="tested">Tested</option><option value="verified">Verified</option>
                          </select>
                          <button onClick={async () => {
                            const updated = [...stories]; updated[si] = storyDraft;
                            setStories(updated); setEditingStory(null); setStoryDraft(null);
                            await fetch('/api/stories', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(updated) })
                          }} style={styles.addBtn}>Save</button>
                          <button onClick={() => { setEditingStory(null); setStoryDraft(null) }} style={styles.cancelBtn}>✕</button>
                        </div>
                      </>
                    ) : (
                      <>
                        {story.steps.map((step, i) => (
                          <div key={i} style={{ padding: '4px 0', fontSize: '0.85rem', display: 'flex', gap: 8 }}>
                            <span style={{ color: '#555', minWidth: 20 }}>{i + 1}.</span>
                            <span style={{ color: '#ccc' }}>{step}</span>
                          </div>
                        ))}
                        {story.notes && <div style={{ marginTop: 8, padding: 8, background: '#1a1a1a', borderRadius: 6, fontSize: '0.8rem', color: '#f59e0b' }}>💡 {story.notes}</div>}
                        <button onClick={() => { setEditingStory(si); setStoryDraft({ ...story }) }} style={{ ...styles.editBtn, marginTop: 8 }}>Edit</button>
                      </>
                    )}
                  </div>
                )}
              </div>
            )
          })}
          <button onClick={async () => {
            const updated = [...stories, { name: 'New user story', steps: ['Step 1'], status: 'draft' as const, notes: '' }];
            setStories(updated);
            await fetch('/api/stories', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(updated) })
          }} style={styles.addComponentBtn}>+ Add story</button>
        </div>
      )}

      {/* SPECS */}
      {tab === 'specs' && (
        <div>
          {specs.map(spec => (
            <div key={spec.name} style={styles.specCard}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
                <h3 style={{ fontSize: '1rem', margin: 0 }}>{spec.filename}</h3>
                {editingSpec === spec.name ? (
                  <div style={{ display: 'flex', gap: 8 }}>
                    <button onClick={() => saveSpec(spec.name)} style={styles.addBtn} disabled={saving}>
                      {saving ? '...' : 'Save'}
                    </button>
                    <button onClick={() => setEditingSpec(null)} style={styles.cancelBtn}>✕</button>
                  </div>
                ) : (
                  <button onClick={() => { setEditingSpec(spec.name); setSpecDraft(spec.content) }} style={styles.editBtn}>Edit</button>
                )}
              </div>
              {editingSpec === spec.name ? (
                <textarea
                  style={styles.specEditor}
                  value={specDraft}
                  onChange={e => setSpecDraft(e.target.value)}
                />
              ) : (
                <pre style={styles.specContent}>{spec.content}</pre>
              )}
            </div>
          ))}
        </div>
      )}

      {/* SKILL.md */}
      {tab === 'skill' && (
        <div style={styles.specCard}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
            <h3 style={{ fontSize: '1rem', margin: 0 }}>SKILL.md — Agent Documentation</h3>
            {editingSkill ? (
              <div style={{ display: 'flex', gap: 8 }}>
                <button onClick={async () => {
                  setSaving(true);
                  await fetch('/api/skill', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: skillDraft }) });
                  setSkillContent(skillDraft); setEditingSkill(false); setSaving(false);
                }} style={styles.addBtn} disabled={saving}>{saving ? '...' : 'Save'}</button>
                <button onClick={() => setEditingSkill(false)} style={styles.cancelBtn}>✕</button>
              </div>
            ) : (
              <button onClick={() => { setEditingSkill(true); setSkillDraft(skillContent) }} style={styles.editBtn}>Edit</button>
            )}
          </div>
          {editingSkill ? (
            <textarea style={{ ...styles.specEditor, minHeight: 600 }} value={skillDraft} onChange={e => setSkillDraft(e.target.value)} />
          ) : (
            <div className="skill-markdown" style={{ padding: '0 4px', lineHeight: 1.6, fontSize: '0.9rem' }}>
              <ReactMarkdown
                components={{
                  code: ({ children, className, ...props }: any) => {
                    const isBlock = className?.startsWith('language-')
                    return isBlock ? (
                      <pre style={{ background: '#1a1a2e', padding: 16, borderRadius: 8, overflowX: 'auto', fontSize: '0.85rem', lineHeight: 1.5 }}>
                        <code style={{ color: '#e0e0e0' }} {...props}>{children}</code>
                      </pre>
                    ) : (
                      <code style={{ background: '#1a1a2e', padding: '2px 6px', borderRadius: 4, fontSize: '0.85em', color: '#7eb6ff' }} {...props}>{children}</code>
                    )
                  },
                  h1: ({ children }) => <h1 style={{ fontSize: '1.4rem', borderBottom: '1px solid #333', paddingBottom: 8, marginTop: 24 }}>{children}</h1>,
                  h2: ({ children }) => <h2 style={{ fontSize: '1.15rem', borderBottom: '1px solid #222', paddingBottom: 6, marginTop: 20, color: '#7eb6ff' }}>{children}</h2>,
                  h3: ({ children }) => <h3 style={{ fontSize: '1rem', marginTop: 16, color: '#a0a0a0' }}>{children}</h3>,
                  table: ({ children }) => <table style={{ borderCollapse: 'collapse', width: '100%', margin: '12px 0' }}>{children}</table>,
                  th: ({ children }) => <th style={{ border: '1px solid #333', padding: '8px 12px', background: '#1a1a2e', textAlign: 'left', fontSize: '0.85rem' }}>{children}</th>,
                  td: ({ children }) => <td style={{ border: '1px solid #333', padding: '8px 12px', fontSize: '0.85rem' }}>{children}</td>,
                  blockquote: ({ children }) => <blockquote style={{ borderLeft: '3px solid #7eb6ff', margin: '12px 0', padding: '8px 16px', color: '#a0a0a0' }}>{children}</blockquote>,
                  ul: ({ children }) => <ul style={{ paddingLeft: 24 }}>{children}</ul>,
                  li: ({ children }) => <li style={{ marginBottom: 4 }}>{children}</li>,
                  a: ({ children, href }) => <a href={href} style={{ color: '#7eb6ff' }} target="_blank" rel="noopener noreferrer">{children}</a>,
                }}
              >{skillContent}</ReactMarkdown>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: { fontFamily: '-apple-system, BlinkMacSystemFont, system-ui, sans-serif', maxWidth: 800, margin: '0 auto', padding: 16, paddingBottom: 100 },
  h1: { fontSize: '1.5rem', marginBottom: 4 },
  subtitle: { color: '#888', fontSize: '0.85rem', marginBottom: 24 },
  tabs: { display: 'flex', gap: 4, marginBottom: 20 },
  tab: { padding: '8px 16px', background: '#1a1a1a', border: '1px solid #333', borderRadius: 8, color: '#aaa', fontSize: '0.85rem', cursor: 'pointer' },
  tabActive: { background: '#2a2a2a', color: '#fff', borderColor: '#555' },
  stats: { display: 'flex', gap: 12, marginBottom: 20, overflowX: 'auto' as const },
  stat: { background: '#1a1a1a', border: '1px solid #222', borderRadius: 12, padding: '12px 16px', minWidth: 80, flexShrink: 0 },
  statValue: { fontSize: '1.5rem', fontWeight: 700 },
  statLabel: { fontSize: '0.7rem', color: '#888', textTransform: 'uppercase' as const, letterSpacing: 0.5 },
  pitch: { background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)', border: '1px solid #2a2a4e', borderRadius: 12, padding: 20, marginBottom: 20, fontSize: '0.9rem', lineHeight: 1.6 },
  code: { background: '#1a1a1a', padding: '1px 5px', borderRadius: 4, fontSize: '0.8rem', color: '#c084fc' },
  sectionTitle: { marginBottom: 12, fontSize: '0.9rem', color: '#888' },
  timelineItem: { position: 'relative' as const, paddingLeft: 24, marginBottom: 20 },
  timelineDot: { position: 'absolute' as const, left: 0, top: 6, width: 12, height: 12, borderRadius: '50%', border: '2px solid #555' },
  timelineDate: { fontSize: '0.7rem', color: '#666', marginBottom: 2 },
  timelineTitle: { fontSize: '0.9rem', fontWeight: 600 },
  timelineDesc: { fontSize: '0.8rem', color: '#999', marginTop: 4 },
  component: { background: '#111', border: '1px solid #222', borderRadius: 12, marginBottom: 12, overflow: 'hidden' },
  componentHeader: { padding: '14px 16px', cursor: 'pointer', display: 'flex', justifyContent: 'space-between', alignItems: 'center' },
  componentName: { fontWeight: 600, fontSize: '0.95rem' },
  badge: { fontSize: '0.7rem', padding: '2px 8px', borderRadius: 10, background: '#1a1a1a', color: '#888', border: '1px solid #333' },
  componentBody: { padding: '0 16px 14px' },
  test: { padding: '8px 0', borderBottom: '1px solid #1a1a1a', display: 'flex', gap: 10, alignItems: 'flex-start' },
  statusBtn: { width: 28, height: 28, border: '1px solid #333', borderRadius: 6, cursor: 'pointer', fontSize: '0.8rem', flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'center' },
  removeBtn: { background: 'none', border: 'none', color: '#555', cursor: 'pointer', fontSize: '1.1rem', padding: '0 4px', flexShrink: 0 },
  input: { flex: 1, background: '#1a1a1a', border: '1px solid #333', borderRadius: 6, color: '#e0e0e0', padding: '6px 10px', fontSize: '0.85rem', outline: 'none' },
  addBtn: { padding: '6px 14px', background: '#052e16', color: '#4ade80', border: '1px solid #166534', borderRadius: 6, cursor: 'pointer', fontSize: '0.8rem' },
  cancelBtn: { padding: '6px 10px', background: '#1a1a1a', color: '#888', border: '1px solid #333', borderRadius: 6, cursor: 'pointer', fontSize: '0.8rem' },
  addTestBtn: { background: 'none', border: '1px dashed #333', borderRadius: 6, color: '#666', padding: '8px 14px', cursor: 'pointer', fontSize: '0.8rem', marginTop: 8, width: '100%' },
  addComponentBtn: { background: 'none', border: '1px dashed #333', borderRadius: 8, color: '#666', padding: '12px', cursor: 'pointer', fontSize: '0.85rem', width: '100%' },
  specCard: { background: '#111', border: '1px solid #222', borderRadius: 12, padding: 16, marginBottom: 12 },
  editBtn: { padding: '4px 12px', background: '#1a1a1a', color: '#aaa', border: '1px solid #333', borderRadius: 6, cursor: 'pointer', fontSize: '0.8rem' },
  specEditor: { width: '100%', minHeight: 400, background: '#0a0a0a', border: '1px solid #333', borderRadius: 8, color: '#e0e0e0', padding: 12, fontFamily: 'monospace', fontSize: '0.8rem', resize: 'vertical' as const, outline: 'none' },
  specContent: { whiteSpace: 'pre-wrap' as const, fontSize: '0.8rem', color: '#bbb', lineHeight: 1.6, margin: 0, fontFamily: 'monospace', maxHeight: 300, overflow: 'auto' },
}
