# Widget: Live Skill Tracking

**ID:** `skill-tracking` | **Default size:** Large | **Component:** `widgets/SkillTrackingWidget.vue`

Grid of `SkillCard` components for skills that have gained XP during the current session:
- Skill name and level (with bonus level breakdown)
- XP gained and XP/hour rate
- Levels gained during session
- Progress bar toward next level
- Estimated time to next level

Shows "No skill updates yet." empty state when no XP has been gained.

**Data source:** `gameStateStore.sessionSkillList`. Session-only, in-memory.
