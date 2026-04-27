---
name: Update docs on every version change
description: User wants DESIGN.md and changelog updated with detailed explanations after every version completion
type: feedback
---

After completing every version update, immediately update DESIGN.md and all relevant docs.
Include detailed explanations for each change, not just bullet points. Explain HOW things
work (like the Swiss Ephemeris explanation), not just WHAT changed.

**Why:** The user wants the design doc to be a complete reference that explains the system
to anyone reading it, including themselves in the future. Technical explanations (like
"Swiss Eph is compiled C code, not an API") belong in the doc.

**How to apply:** After finishing any version milestone (v2.0.1, v2.0.2, etc.):
1. Update the version number in the DESIGN.md header
2. Add a detailed changelog entry with theme, explanation of what changed and WHY
3. Update the architecture tree if new files were created
4. Update the migration list
5. Update any formulas/tables that changed
6. Move completed backlog items to the Completed section
7. Include enough technical detail that someone new could understand the system
