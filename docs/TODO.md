# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-03*

---

## To Sort

- BUG: Market Prices screen doesn't scroll properly.
- IMPROVEMENT: Market Prices screen needs better layout.
- IMPROVEMENT: Market Prices screen needs filtering.
- IMPROVEMENT: Some way to bulk set prices?

---


## Medium Effort, High Value

- [ ] Update data browser to have better, more readable layouts
  - Design/UX task. All tabs already use `PaneLayout` with consistent two-pane search/detail pattern. Improvements are incremental polish: better section headers, consistent key-value grids, spacing per `docs/architecture/ux-standards.md`. Could break into sub-tasks per browser tab.
  - **Effort: Medium (iterative) | Impact: Medium (polish)**

---

## Larger Efforts / Research Needed

- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Gear transmutation reference tool (reported by Reyetta)
  - No transmutation data in CDN. Would need manual data entry for transmutation rules, costs, and outcomes. Code is straightforward once data exists — the data is the bottleneck.
  - **Effort: Large (data acquisition) | Impact: Low-Medium (niche use case)**
