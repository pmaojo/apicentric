## 2024-05-23 - Custom Hook Signature Mismatch
**Learning:** The `useWebSocketSubscription` custom hook in this codebase accepts exactly two arguments (event name and callback). Passing a dependency array as a third argument (standard React pattern) causes type errors and potential runtime issues.
**Action:** Always verify custom hook signatures in `providers/` or `hooks/` before usage, do not assume standard React hook signatures apply to custom ones.
