## 2026-01-27 - React Virtualization & Memoization
**Learning:** `LogsViewer` used `useVirtualizer` but rendered rows inline. This negates some benefits as the parent re-render (due to log updates) causes all virtual rows to re-render, creating new DOM nodes (or diffing them). Extracting the row to a memoized component is critical for high-frequency updates (logs).
**Action:** Always extract row components in virtualized lists, especially if the parent component updates frequently. Use `React.memo` and ensure props (like event handlers) are stable.

## 2026-01-27 - Next.js Linting Environment
**Learning:** The repository lacks `eslint` and `eslint-config-next` in `package.json` despite having `next lint` script. Attempting to run it fails.
**Action:** Rely on `tsc --noEmit` (typecheck) for validation if linting environment is broken, rather than trying to fix the environment (which is out of scope/risky).
