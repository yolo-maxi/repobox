(globalThis.TURBOPACK||(globalThis.TURBOPACK=[])).push(["object"==typeof document?document.currentScript:void 0,95057,(e,t,r)=>{"use strict";Object.defineProperty(r,"__esModule",{value:!0});var n={formatUrl:function(){return l},formatWithValidation:function(){return u},urlObjectKeys:function(){return i}};for(var a in n)Object.defineProperty(r,a,{enumerable:!0,get:n[a]});let o=e.r(90809)._(e.r(98183)),s=/https?|ftp|gopher|file/;function l(e){let{auth:t,hostname:r}=e,n=e.protocol||"",a=e.pathname||"",l=e.hash||"",i=e.query||"",u=!1;t=t?encodeURIComponent(t).replace(/%3A/i,":")+"@":"",e.host?u=t+e.host:r&&(u=t+(~r.indexOf(":")?`[${r}]`:r),e.port&&(u+=":"+e.port)),i&&"object"==typeof i&&(i=String(o.urlQueryToSearchParams(i)));let c=e.search||i&&`?${i}`||"";return n&&!n.endsWith(":")&&(n+=":"),e.slashes||(!n||s.test(n))&&!1!==u?(u="//"+(u||""),a&&"/"!==a[0]&&(a="/"+a)):u||(u=""),l&&"#"!==l[0]&&(l="#"+l),c&&"?"!==c[0]&&(c="?"+c),a=a.replace(/[?#]/g,encodeURIComponent),c=c.replace("#","%23"),`${n}${u}${a}${c}${l}`}let i=["auth","hash","host","hostname","href","path","pathname","port","protocol","query","search","slashes"];function u(e){return l(e)}},18581,(e,t,r)=>{"use strict";Object.defineProperty(r,"__esModule",{value:!0}),Object.defineProperty(r,"useMergedRef",{enumerable:!0,get:function(){return a}});let n=e.r(71645);function a(e,t){let r=(0,n.useRef)(null),a=(0,n.useRef)(null);return(0,n.useCallback)(n=>{if(null===n){let e=r.current;e&&(r.current=null,e());let t=a.current;t&&(a.current=null,t())}else e&&(r.current=o(e,n)),t&&(a.current=o(t,n))},[e,t])}function o(e,t){if("function"!=typeof e)return e.current=t,()=>{e.current=null};{let r=e(t);return"function"==typeof r?r:()=>e(null)}}("function"==typeof r.default||"object"==typeof r.default&&null!==r.default)&&void 0===r.default.__esModule&&(Object.defineProperty(r.default,"__esModule",{value:!0}),Object.assign(r.default,r),t.exports=r.default)},73668,(e,t,r)=>{"use strict";Object.defineProperty(r,"__esModule",{value:!0}),Object.defineProperty(r,"isLocalURL",{enumerable:!0,get:function(){return o}});let n=e.r(18967),a=e.r(52817);function o(e){if(!(0,n.isAbsoluteUrl)(e))return!0;try{let t=(0,n.getLocationOrigin)(),r=new URL(e,t);return r.origin===t&&(0,a.hasBasePath)(r.pathname)}catch(e){return!1}}},84508,(e,t,r)=>{"use strict";Object.defineProperty(r,"__esModule",{value:!0}),Object.defineProperty(r,"errorOnce",{enumerable:!0,get:function(){return n}});let n=e=>{}},22016,(e,t,r)=>{"use strict";Object.defineProperty(r,"__esModule",{value:!0});var n={default:function(){return y},useLinkStatus:function(){return x}};for(var a in n)Object.defineProperty(r,a,{enumerable:!0,get:n[a]});let o=e.r(90809),s=e.r(43476),l=o._(e.r(71645)),i=e.r(95057),u=e.r(8372),c=e.r(18581),p=e.r(18967),d=e.r(5550);e.r(33525);let f=e.r(88540),g=e.r(91949),h=e.r(73668),m=e.r(9396);function y(t){var r,n;let a,o,y,[x,v]=(0,l.useOptimistic)(g.IDLE_LINK_STATUS),w=(0,l.useRef)(null),{href:A,as:E,children:C,prefetch:j=null,passHref:T,replace:k,shallow:N,scroll:O,onClick:S,onMouseEnter:B,onTouchStart:R,legacyBehavior:_=!1,onNavigate:P,transitionTypes:L,ref:I,unstable_dynamicOnHover:M,...$}=t;a=C,_&&("string"==typeof a||"number"==typeof a)&&(a=(0,s.jsx)("a",{children:a}));let D=l.default.useContext(u.AppRouterContext),U=!1!==j,F=!1!==j?null===(n=j)||"auto"===n?m.FetchStrategy.PPR:m.FetchStrategy.Full:m.FetchStrategy.PPR,Y="string"==typeof(r=E||A)?r:(0,i.formatUrl)(r);if(_){if(a?.$$typeof===Symbol.for("react.lazy"))throw Object.defineProperty(Error("`<Link legacyBehavior>` received a direct child that is either a Server Component, or JSX that was loaded with React.lazy(). This is not supported. Either remove legacyBehavior, or make the direct child a Client Component that renders the Link's `<a>` tag."),"__NEXT_ERROR_CODE",{value:"E863",enumerable:!1,configurable:!0});o=l.default.Children.only(a)}let K=_?o&&"object"==typeof o&&o.ref:I,H=l.default.useCallback(e=>(null!==D&&(w.current=(0,g.mountLinkInstance)(e,Y,D,F,U,v)),()=>{w.current&&((0,g.unmountLinkForCurrentNavigation)(w.current),w.current=null),(0,g.unmountPrefetchableInstance)(e)}),[U,Y,D,F,v]),W={ref:(0,c.useMergedRef)(H,K),onClick(t){_||"function"!=typeof S||S(t),_&&o.props&&"function"==typeof o.props.onClick&&o.props.onClick(t),!D||t.defaultPrevented||function(t,r,n,a,o,s,i){if("u">typeof window){let u,{nodeName:c}=t.currentTarget;if("A"===c.toUpperCase()&&((u=t.currentTarget.getAttribute("target"))&&"_self"!==u||t.metaKey||t.ctrlKey||t.shiftKey||t.altKey||t.nativeEvent&&2===t.nativeEvent.which)||t.currentTarget.hasAttribute("download"))return;if(!(0,h.isLocalURL)(r)){a&&(t.preventDefault(),location.replace(r));return}if(t.preventDefault(),s){let e=!1;if(s({preventDefault:()=>{e=!0}}),e)return}let{dispatchNavigateAction:p}=e.r(99781);l.default.startTransition(()=>{p(r,a?"replace":"push",!1===o?f.ScrollBehavior.NoScroll:f.ScrollBehavior.Default,n.current,i)})}}(t,Y,w,k,O,P,L)},onMouseEnter(e){_||"function"!=typeof B||B(e),_&&o.props&&"function"==typeof o.props.onMouseEnter&&o.props.onMouseEnter(e),D&&U&&(0,g.onNavigationIntent)(e.currentTarget,!0===M)},onTouchStart:function(e){_||"function"!=typeof R||R(e),_&&o.props&&"function"==typeof o.props.onTouchStart&&o.props.onTouchStart(e),D&&U&&(0,g.onNavigationIntent)(e.currentTarget,!0===M)}};return(0,p.isAbsoluteUrl)(Y)?W.href=Y:_&&!T&&("a"!==o.type||"href"in o.props)||(W.href=(0,d.addBasePath)(Y)),y=_?l.default.cloneElement(o,W):(0,s.jsx)("a",{...$,...W,children:a}),(0,s.jsx)(b.Provider,{value:x,children:y})}e.r(84508);let b=(0,l.createContext)(g.IDLE_LINK_STATUS),x=()=>(0,l.useContext)(b);("function"==typeof r.default||"object"==typeof r.default&&null!==r.default)&&void 0===r.default.__esModule&&(Object.defineProperty(r.default,"__esModule",{value:!0}),Object.assign(r.default,r),t.exports=r.default)},83047,e=>{"use strict";e.i(47167);var t=e.i(43476),r=e.i(71645),n=e.i(22016);let a=`You generate .repobox.yml files. This is a PROPRIETARY format for repo.box, a git permission layer for AI agents. Do NOT invent fields or use any other YAML schema.

THE EXACT AND ONLY FORMAT:

groups:
  groupname:
    - evm:0xADDRESS...
    - other-group-name

permissions:
  default: allow
  rules: ...  # THREE valid formats (see below)

GROUPS: A flat list of evm:0x... addresses directly under the group name. NO "members:" key. Can include another group by bare name.

THREE RULE FORMATS (all equivalent, mix freely):

Format A — Flat list (rules is a YAML list of strings):
  rules:
    - "groupname verb target"
    - "groupname not verb target"

Format B — Subject-grouped (rules is a mapping, subject → list of "verb target" strings):
  rules:
    groupname:
      - "verb target"
      - "not verb target"

Format C — Verb-mapping (rules is a mapping, subject → verb → targets):
  rules:
    groupname:
      verb:
        - "target"

In Format A, entries can also be nested mappings (C-style inside a list):
  rules:
    - "groupname verb target"
    - groupname:
        verb:
          - "target"

Use whichever feels natural. All three parse identically.

SUBJECTS in rules: bare group names (founders, agents) or evm:0x... addresses. NO prefix (no %, no @).

BRANCH VERBS: push, merge, create, delete, force-push
FILE VERBS: edit (full modify), write (add-only), append (end-only)
DENY: "groupname not verb target"

TARGETS:
  >main — exact branch
  >feature/** — branch glob (recursive)
  >* — all branches
  * — all files (when used with file verbs)
  ./* — all files (equivalent to *)
  ./contracts/** — file path glob (./prefix optional but recommended)
  ./.repobox.yml — the config file
  ./contracts/** >dev — file + branch combo

CRITICAL RULES:
- Groups are flat lists (no "members:" key)
- Group names in rules are bare words: founders, agents (NO %, NO @)
- File paths use ./ prefix (optional but recommended): ./.repobox.yml, ./* >feature/**
- Branch verbs use targets starting with >
- File verbs use ./ file path targets
- TWO INDEPENDENT checks — branch ops AND file ops must both pass
- default: allow = unmentioned verb+target combos are permitted
- default: deny = everything not explicitly allowed is denied
- Implicit deny per target: if ANY rule mentions a verb+target, others are denied for THAT target
- Top-to-bottom priority: first match wins
- Quote targets starting with > in nested YAML rules
- Use placeholder addresses: evm:0xAAA...111, evm:0xBBB...222, evm:0xCCC...333, etc.

COMPLETE EXAMPLE (team + AI agents):

groups:
  founders:
    - evm:0xAAA...111
    - evm:0xBBB...222
  agents:
    - evm:0xCCC...333
    - evm:0xDDD...444

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders edit ./.repobox.yml
    - agents:
        push:
          - ">feature/**"
          - ">fix/**"
        create:
          - ">feature/**"
          - ">fix/**"
        append:
          - "./.repobox.yml"

This config: founders can do anything. Agents can only push/create feature and fix branches. Only founders can edit .repobox.yml (agents can append). Since default is allow and no edit rules exist for source files, anyone can edit any file.

ANOTHER EXAMPLE (file lockdown):

groups:
  maintainers:
    - evm:0xAAA...111
  contributors:
    - evm:0xBBB...222

permissions:
  default: allow
  rules:
    - maintainers push >*
    - maintainers merge >*
    - maintainers create >*
    - maintainers edit ./contracts/**
    - maintainers edit ./.repobox.yml
    - contributors:
        push:
          - ">contributor/**"
        create:
          - ">contributor/**"

This config: maintainers can push/merge/create any branch and edit contracts + config. Contributors can only push/create contributor/* branches. Contracts folder is implicitly denied to contributors. Other files are open (default: allow).

ANOTHER EXAMPLE (strict with file control):

groups:
  founders:
    - evm:0xAAA...111
  agents:
    - evm:0xBBB...222

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders edit ./.repobox.yml
    - agents:
        push:
          - ">feature/**"
          - ">fix/**"
        create:
          - ">feature/**"
          - ">fix/**"
        edit:
          - "./* >feature/**"
          - "./* >fix/**"
        append:
          - "./.repobox.yml"

This config: agents can edit files ONLY on their branches (./* >feature/** means all files but only on feature branches).

OUTPUT RULES:
- When GENERATING: output ONLY the raw YAML. No markdown fences, no explanation, no comments, no \`\`\` wrappers.
- When EXPLAINING: describe what each group can and cannot do in plain English. Use bullet points. Be specific about allowed vs denied actions.`,o=[`groups:
  founders:
    - evm:0xAAA...123
  agents:
    - evm:0xBBB...456

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - agents push >feature/**
    - agents create >feature/**`,`groups:
  founders:
    - evm:0xAAA...123
  agents:
    - evm:0xBBB...456
    - evm:0xCCC...789

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders edit ./.repobox.yml
    - agents:
        push:
          - ">feature/**"
          - ">fix/**"
        create:
          - ">feature/**"
          - ">fix/**"
        edit:
          - "./* >feature/**"
          - "./* >fix/**"
        append:
          - "./.repobox.yml"`,`groups:
  orchestrator:
    - evm:0xAAA...001
  workers:
    - evm:0xBBB...002
    - evm:0xCCC...003
    - evm:0xDDD...004
    - evm:0xEEE...005

permissions:
  default: deny
  rules:
    - orchestrator push >*
    - orchestrator merge >*
    - orchestrator create >*
    - orchestrator edit *
    - workers not force-push >*
    - workers:
        push:
          - ">worker-1/**"
          - ">worker-2/**"
          - ">worker-3/**"
          - ">worker-4/**"
        create:
          - ">worker-1/**"
          - ">worker-2/**"
          - ">worker-3/**"
          - ">worker-4/**"
        edit:
          - "./* >worker-1/**"
          - "./* >worker-2/**"
          - "./* >worker-3/**"
          - "./* >worker-4/**"
        append:
          - "./.repobox.yml"`],s=[{label:"team + agents",text:"Two founders with full access. Three AI agents that can only work on feature branches. Nobody except founders can touch the config file."},{label:"solo + codex",text:"Solo developer. One Codex agent that can push to any branch except main, and can only append to the config."},{label:"open source",text:"Open source repo. Maintainers can merge to main. External contributors can only push to their own branches (contributor/*). Contracts folder is locked to maintainers."},{label:"multi-agent",text:"Five agents, five keys, one repo. Each agent gets its own feature namespace. Only the orchestrator agent can merge. Nobody can force-push."}],l=["minimal","file-locked","multi-agent"];e.s(["PlaygroundClient",0,function(){let[e,i]=(0,r.useState)("generate"),[u,c]=(0,r.useState)(""),[p,d]=(0,r.useState)(""),[f,g]=(0,r.useState)(""),[h,m]=(0,r.useState)(!1),[y,b]=(0,r.useState)(""),x=(0,r.useRef)(null),v=(0,r.useRef)(null),w=(0,r.useCallback)(async()=>{let t="generate"===e?u.trim():p.trim();if(!t)return;x.current&&x.current.abort();let r=new AbortController;x.current=r,g(""),m(!0),b("thinking...");let n="generate"===e?`Generate a .repobox.yml for this scenario:

${t}

Output ONLY the YAML. No explanation, no code fences.`:`Explain this .repobox.yml in plain English. What can each group do? What are they denied?

${t}`;try{let e=await fetch("https://api.venice.ai/api/v1/chat/completions",{method:"POST",headers:{"Content-Type":"application/json",Authorization:"Bearer c8dgN2WwVdKjYheUepXmm2aa5X31X08CF6SIicy3a9"},body:JSON.stringify({model:"qwen3-235b-a22b-instruct-2507",stream:!0,messages:[{role:"system",content:a},{role:"user",content:n}],temperature:.3,max_tokens:2e3,venice_parameters:{include_venice_system_prompt:!1,disable_thinking:!0}}),signal:r.signal});if(!e.ok)throw Error(`API error: ${e.status}`);b("streaming...");let t=e.body.getReader(),o=new TextDecoder,s="",l="";for(;;){let{done:e,value:r}=await t.read();if(e)break;let n=(s+=o.decode(r,{stream:!0})).split("\n");for(let e of(s=n.pop(),n)){if(!e.startsWith("data: "))continue;let t=e.slice(6).trim();if("[DONE]"!==t)try{let e=JSON.parse(t),r=e.choices?.[0]?.delta?.content;r&&(l+=r,g(l),v.current&&(v.current.scrollTop=v.current.scrollHeight))}catch{}}}}catch(e){"AbortError"!==e.name&&g(`Error: ${e.message}`)}finally{m(!1),b(""),x.current=null}},[e,u,p]),A=(0,r.useCallback)(()=>{let e=v.current?.innerText||f;navigator.clipboard.writeText(e)},[f]),E=(0,r.useCallback)(e=>{(e.ctrlKey||e.metaKey)&&"Enter"===e.key&&(e.preventDefault(),w())},[w]),C=f?"generate"===e?f.replace(/^```ya?ml\n?/gm,"").replace(/^```\n?/gm,"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/(#.*$)/gm,'<span style="color:#3a5a72;">$1</span>').replace(/^(\s*)([\w-]+)(:)/gm,'$1<span style="color:#4fc3f7;">$2</span><span style="color:#5a7a94;">$3</span>').replace(/(&quot;[^&]*&quot;|'[^']*')/g,'<span style="color:#81d4fa;">$1</span>').replace(/^(\s*)(- )/gm,'$1<span style="color:#5a7a94;">- </span>').replace(/(evm:0x[\w.]+)/g,'<span style="color:#b8d4e3;opacity:0.7;">$1</span>').replace(/(\.\/[\w.*\/-]+)/g,'<span style="color:#81d4fa;">$1</span>'):f.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/\*\*([^*]+)\*\*/g,'<span style="color:#e8f4fd;font-weight:600;">$1</span>').replace(/`([^`]+)`/g,'<span style="color:#4fc3f7;background:rgba(79,195,247,0.08);padding:0 4px;border-radius:2px;">$1</span>').replace(/^(\s*[-•])/gm,'<span style="color:#4fc3f7;">$1</span>').replace(/✅/g,'<span style="color:#4caf50;">✅</span>').replace(/❌/g,'<span style="color:#f44336;">❌</span>'):"";return(0,t.jsxs)("div",{className:"max-w-[760px] mx-auto",style:{padding:"40px 20px 100px"},onKeyDown:E,children:[(0,t.jsxs)("header",{style:{marginBottom:48},children:[(0,t.jsxs)("div",{style:{display:"flex",alignItems:"baseline",justifyContent:"space-between",marginBottom:24},children:[(0,t.jsxs)(n.default,{href:"/",style:{fontWeight:700,fontSize:24,lineHeight:1.1},children:["repo",(0,t.jsx)("span",{className:"logo-dot",children:"."}),"box"]}),(0,t.jsx)("span",{style:{fontSize:12,color:"var(--bp-dim)",textTransform:"uppercase",letterSpacing:"0.12em"},children:"playground"})]}),(0,t.jsxs)("p",{style:{fontSize:14,lineHeight:"22px",color:"var(--bp-dim)",maxWidth:560},children:["Try the config generator. Describe your repo permissions in English and get a"," ",(0,t.jsx)("code",{style:{color:"var(--bp-accent)",background:"rgba(79,195,247,0.08)",padding:"1px 6px",borderRadius:3},children:".repobox.yml"})," ","— or paste a config to understand what it does."]})]}),(0,t.jsxs)("div",{style:{display:"flex",gap:8,marginBottom:24},children:[(0,t.jsx)("button",{className:`playground-mode-btn ${"generate"===e?"active":""}`,onClick:()=>i("generate"),children:"English → Config"}),(0,t.jsx)("button",{className:`playground-mode-btn ${"explain"===e?"active":""}`,onClick:()=>i("explain"),children:"Config → English"})]}),"generate"===e&&(0,t.jsxs)("div",{children:[(0,t.jsx)("label",{className:"playground-label",children:"Describe your permissions"}),(0,t.jsx)("textarea",{className:"playground-textarea",rows:6,value:u,onChange:e=>c(e.target.value),placeholder:"e.g. Three founders can do anything. Two AI agents can only push to feature branches and can't touch the config file."}),(0,t.jsx)("div",{className:"playground-chips",children:s.map(e=>(0,t.jsx)("button",{className:"playground-chip",onClick:()=>c(e.text),children:e.label},e.label))})]}),"explain"===e&&(0,t.jsxs)("div",{children:[(0,t.jsx)("label",{className:"playground-label",children:"Paste a .repobox.yml"}),(0,t.jsx)("textarea",{className:"playground-textarea",rows:10,value:p,onChange:e=>d(e.target.value),placeholder:`groups:
  founders:
    - evm:0xAAA...
  agents:
    - evm:0xBBB...

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - agents push >feature/**
    - agents create >feature/**`}),(0,t.jsx)("div",{className:"playground-chips",children:l.map((e,r)=>(0,t.jsx)("button",{className:"playground-chip",onClick:()=>d(o[r]),children:e},e))})]}),(0,t.jsxs)("div",{style:{display:"flex",alignItems:"center",gap:12,marginBottom:8},children:[(0,t.jsx)("button",{className:"playground-go-btn",onClick:w,disabled:h,children:"generate"===e?"Generate":"Explain"}),y&&(0,t.jsxs)("span",{style:{fontSize:11,color:"var(--bp-dim)",display:"inline-flex",alignItems:"center",gap:6},children:[(0,t.jsx)("span",{style:{width:6,height:6,borderRadius:"50%",background:"var(--bp-accent)",animation:"pulse 2s infinite",display:"inline-block"}}),y]})]}),(0,t.jsx)("div",{style:{display:"flex",alignItems:"center",justifyContent:"center",color:"#2a4a62",fontSize:20,padding:"8px 0"},children:"↓"}),(0,t.jsxs)("div",{style:{display:"flex",alignItems:"baseline",justifyContent:"space-between",marginBottom:8},children:[(0,t.jsx)("label",{className:"playground-label",children:"generate"===e?"Generated config":"Explanation"}),(0,t.jsx)("button",{className:"playground-copy-btn",onClick:A,children:"copy"})]}),(0,t.jsx)("div",{ref:v,className:`playground-output ${h?"streaming":""}`,dangerouslySetInnerHTML:C?{__html:C}:{__html:'<span style="color:#2a4a62;">Output will appear here...</span>'}})]})}],83047)}]);