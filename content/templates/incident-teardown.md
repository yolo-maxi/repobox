# Incident Teardown Template

## Blog Post Template for Security Incident Analysis

**Filename format:** `YYYY-MM-DD-[company]-[incident-type]-teardown.md`  
**Example:** `2026-03-30-circleci-token-breach-teardown.md`

---

```markdown
---
title: "[Company] [Incident Type] Teardown: [Brief Description]"
date: YYYY-MM-DD
description: "Technical analysis of [company]'s [incident] showing [key failure types] and how repo.box prevents this class of attack."
tags: [security, incident-response, access-control, [company-tag]]
---

> **Context:** Brief incident overview (1-2 sentences). When did it happen, what was compromised, scope of impact.

## Incident Summary

**System Affected:** [Product/Service/Infrastructure Component]  
**Failure Type:** [Access Control / Supply Chain / Token Compromise / etc.]  
**Discovery Date:** [Date]  
**Resolution Date:** [Date]  
**Total Duration:** [X hours/days]

### Timeline

- **[Date/Time]** - Initial compromise/vulnerability introduced
- **[Date/Time]** - Exploitation begins (if known)
- **[Date/Time]** - Discovery and initial response  
- **[Date/Time]** - Full scope understood
- **[Date/Time]** - Mitigation deployed
- **[Date/Time]** - Resolution confirmed

## Access Control Analysis

### Permission Failures

[Detailed technical analysis of how permissions were bypassed, escalated, or abused]

**Root Causes:**
- **Overprivileged Tokens:** [Description of token scope issues]
- **Role Escalation:** [How attackers gained elevated privileges] 
- **Policy Gaps:** [Missing or inadequate access controls]

### Attack Vector Diagram

<div class="visual-break">
<pre class="diagram">
[ASCII diagram showing the attack flow through systems]
Example:
Developer Machine → Token Extraction → CI/CD Access → Production Deploy
      ↓                     ↓              ↓              ↓
   [Weakness 1]         [Weakness 2]   [Weakness 3]   [Impact]
</pre>
</div>

## Blast Radius Assessment

### Affected Systems
- **[System Category]:** [Impact description]
- **[System Category]:** [Impact description]

### Data Exposure
- **Type:** [Customer data/secrets/source code/etc.]
- **Volume:** [Estimated records/accounts affected]
- **Sensitivity:** [Classification and risk level]

### Service Downtime
- **Duration:** [Total service impact time]
- **Severity:** [Complete outage/degraded performance/etc.]
- **User Impact:** [Description of customer-facing effects]

## Resolution & Prevention

### Immediate Response
- **Containment:** [Actions taken to stop the attack]
- **Evidence Preservation:** [Forensic measures]
- **Communication:** [Internal/external notification timeline]

### Long-term Fixes
- **Technical Changes:** [Infrastructure/code changes made]
- **Policy Updates:** [New procedures and controls]
- **Monitoring Improvements:** [Enhanced detection capabilities]

### Lessons Learned
- [Key insight 1]
- [Key insight 2] 
- [Key insight 3]

## repo.box Prevention Architecture

### How We Prevent This Attack Class

**[Specific Technical Approach]:**
[Detailed explanation of how repo.box's architecture specifically prevents this type of incident]

**[Key Security Feature]:**
[Description of relevant security mechanism]

### Architecture Comparison

<div class="visual-break">
<pre class="diagram">
TRADITIONAL APPROACH          REPO.BOX APPROACH
───────────────────          ─────────────────
[Vulnerable pattern]   →      [Secure pattern]
[Risk point 1]        →      [Mitigation 1]
[Risk point 2]        →      [Mitigation 2]
</pre>
</div>

### Technical Implementation

```markdown
Key security boundaries that prevent this attack:
- **[Boundary 1]:** [Technical description]  
- **[Boundary 2]:** [Technical description]
- **[Boundary 3]:** [Technical description]
```

## Industry Impact

### Broader Implications
[Discussion of what this incident means for the industry/ecosystem]

### Similar Incidents
- **[Related incident 1]:** [Brief comparison]
- **[Related incident 2]:** [Brief comparison]

### Pattern Recognition
[Analysis of recurring themes across similar incidents]

## Conclusion

[Summary of key takeaways and how repo.box addresses the fundamental issues]

**The Bottom Line:** [One-sentence summary of the core problem and solution]

---

*Learn more about repo.box security architecture at [/trust](https://repo.box/trust) or explore our [package ecosystem](/packages).*
```

---

## Template Usage Notes

### SEO Target Keywords
- "[Company] security breach analysis"
- "access control failure prevention" 
- "incident response lessons"
- "[Incident type] prevention"
- "[Company] [year] incident analysis"

### Social Sharing Requirements
- Custom incident-specific imagery (diagrams, architecture comparisons)
- Twitter/LinkedIn optimized descriptions
- Incident-specific meta tags

### Content Guidelines
- **Length:** 1200-1500 words
- **Technical Depth:** Engineering audience appropriate
- **Tone:** Analytical, not inflammatory
- **Evidence:** Only publicly documented details
- **CTA:** Always end with repo.box value proposition

### Exclusions
- No speculation on unreported details
- No client-confidential information  
- No inflammatory criticism of affected companies
- No unfounded technical claims

---

## Publishing Checklist

- [ ] All technical details verified from public sources
- [ ] Architecture diagrams created and optimized
- [ ] SEO metadata and social sharing tags added
- [ ] Mobile-responsive design verified
- [ ] Added to RSS feed
- [ ] Cross-promotion integration ready
- [ ] repo.box CTA links functional (/trust, /packages)