# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-03*

---

## To Sort

- BUG: Market Prices screen doesn't scroll properly.
- IMPROVEMENT: Market Prices screen needs better layout.
- IMPROVEMENT: Market Prices screen needs filtering.
- IMPROVEMENT: Some way to bulk set prices?
- IMPROVEMENT: better support in build planner for using recipes that consume crafting points (right now we really only support augments)
- IMPROVEMENT: belts in the build planner are kinda busted. need a systematic way to handle belts
- IMPROVEMENT: build planner layout needs to use our PanelLayout
- IMPROVEMENT: Save page state of projects page when navigating off it.
- IDEA/BRAINSTORM: How do we improve the UI/UX of going to check the recipe of something, etc then going back to what you were doing? Should the whole databrowser live as a popover? Think on this UX
- IDEA: Quick reference favorites somehow? a panel? popup from bottom toolbar? something? what's our version of solving the players with multiple wiki tabs open. 
- IMPROVEMENT: Need better formatting for older chat lines that include date.
- IMPROVEMENT: Primary/Secondary naming on gear is confusing in the build planner. We can make this more descriptive.
- IMPROVEMENT: Actually implement toast notifications and audio alerts for watchwords
- IMPROVEMENT: show latest detections of watchwords on the dashboard
- IMPROVEMENT: show rez timer on the dashboard
- IMPROVEMENT: show critical resources (diamonds, amethysts, aquamarines, eternal greens, salt, fire dust) counts in inventory on the dashboard.

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
