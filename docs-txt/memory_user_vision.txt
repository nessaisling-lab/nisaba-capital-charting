---
name: Product vision and design philosophy
description: User wants Bloomberg-class Rust terminal inspired by FinceptTerminal, Buffett philosophy, and Apple UX
type: project
---

The user's vision is to build toward a Bloomberg-class native Rust financial terminal,
drawing inspiration from three sources:

1. **FinceptTerminal** (C++20/Qt6) — AI agent personas (Buffett, Graham, Lynch, Munger),
   100+ data connectors, QuantLib analytics, node editor for visual workflows, 16 broker
   integrations, CFA-level analytics. User wants to port as much as feasible to Rust/Iced.

2. **Warren Buffett philosophy** — patient value investing, margin of safety, circle of
   competence, long-term compounding, skepticism of complexity, "be fearful when others
   are greedy." The dashboard should embody this: clear signal over noise, honest about
   what data shows, no hype.

3. **Apple UX/UI philosophy** — clarity, deference, depth. Information density without
   visual clutter. Every element earns its place. Typography and spacing as primary design
   tools. Light/dark theme done right. Progressive disclosure.

**Why:** This context should shape all future feature and design decisions.

**How to apply:** When adding new panels, features, or visuals, filter through all three
lenses: Does Fincept have something similar we can learn from? Would Buffett find this
useful or distracting? Does this meet Apple's bar for clarity and polish?
