# Toast System

Transient notification system for non-blocking user feedback.

## Architecture

Three layers:

1. **Store** — [src/stores/toastStore.ts](../../src/stores/toastStore.ts) — Pinia store holding a reactive array of toast objects. Not persisted.
2. **Composable** — [src/composables/useToast.ts](../../src/composables/useToast.ts) — Convenience wrapper that screens import.
3. **Container** — [src/components/Shared/ToastContainer.vue](../../src/components/Shared/ToastContainer.vue) — Renders visible toasts. Mounted once in App.vue via Teleport.

## Toast Types

| Type | Left Border | Prefix | Auto-dismiss |
|------|------------|--------|-------------|
| success | `accent-green` | ✓ | 4 seconds |
| info | `accent-blue` | ● | 4 seconds |
| warning | `accent-warning` | ▲ | 4 seconds |
| error | `accent-red` | ✕ | No (manual dismiss) |

## Behavior

- Toasts appear in the **bottom-right** corner, stacked vertically with newest on top
- Maximum **3 visible** at a time. Overflow auto-dismisses the oldest.
- **Hover** pauses the auto-dismiss timer
- **Esc** dismisses the top toast (via `useKeyboard` fallback)
- Slide-in animation from the right

## Usage

```
const toast = useToast()
toast.success("Character imported")
toast.error("Failed to load game data")
```

## When to Use Toasts vs Inline Feedback

- **Toast:** action completed, background operation finished, non-critical warnings — things the user should know but doesn't need to act on immediately
- **Inline:** validation errors, loading states, empty states — things the user needs to see *in context* to take their next action

Rule of thumb: if the feedback is about *the thing the user is looking at*, put it inline. If it's about *something that happened elsewhere*, use a toast.
