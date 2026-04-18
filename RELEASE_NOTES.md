## What's Changed since v0.6.1

### Features
- feat: add mushroom farming dashboard widget (`75b0e69`)
- feat: add teleport machine codes dashboard widget (`2602bb3`)
- feat: item parser 2.0 + surveys 2.0 (`7edd681`)

### Improvements
- impv: better batching and sleep controls for dual-log replays for development (`1ec25a1`)
- impv: show next moon phase instead of only full moon in mushroom widget (`fce2630`)
- impv: polish teleport codes and mushroom farming widgets (`bfd4a9d`)
- impv: cap zone NPCs widget height to prevent it dominating dashboard (`fb6e759`)
- impv: recipes widget now checks both inventory and storage (`b87a4b0`)
- impv: improve text contrast and bump base font size (`f1ad9f7`)
- impv: auto-expand NPC details panel when selecting an NPC (`2ab182f`)

### Fixes
- fix: forgot one piece of unhooking the tracker (`abdb612`)
- fix: recipe widget properly looks up recipe requirements (`3688959`)
- fix: right sidebar in the survey tracker starts expanded (`31f6dfc`)
- fix: projects now respect the sort rules of that list (`08aae15`)
- fix: removing the 'start tracking' button since it is broke as hell. hardly a fix, but added a todo to investigate and reimplement later (`d226e94`)
- fix: stampede correctly shows with proper ranks in build planner and browser fix: we now rebuild derived tables on version change. (`2fe3aa6`)
- fix: farming session timer handles midnight rollover (`a31399a`)
- fix: pricing calculator now uses market values with value*2 fallback (`bbd6a7c`)
- fix: ability families with mixed monster/player tiers no longer misclassified (`6616540`)
- fix: build planner name dropdown now gets its own line (`54f3fbc`)
- fix: tooltips now flip above/below to stay within viewport (`2bcb1e9`)

---
*23 commits since v0.6.1*
