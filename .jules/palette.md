# Palette's Journal

## 2024-05-23 - [Initial Entry]
**Learning:** This journal tracks critical UX and accessibility learnings.
**Action:** Use this file to record insights that can guide future design decisions.

## 2024-05-23 - [Keyboard Accessibility in Virtualized Lists]
**Learning:** Virtualized lists (like log viewers) often break accessibility because they render `div`s instead of native buttons for performance/layout reasons. This makes them invisible to keyboard users.
**Action:** Always wrap interactive list items in a generic `div` with `role="button"`, `tabIndex={0}`, and an `onKeyDown` handler for Enter/Space. Ensure visual focus states are present.

## 2024-05-24 - [Invisible Copy Buttons & Keyboard Focus]
**Learning:** Action buttons that appear only on hover (using `opacity-0 group-hover:opacity-100`) are invisible to keyboard users even when they receive focus.
**Action:** Always include `focus:opacity-100` alongside hover classes to ensure keyboard users can see what they are focusing on.
