## 2026-01-28 - Regex Cache Optimization in Simulator
**Learning:** Compiling regexes inside a request loop (O(N) where N is endpoints) is a significant bottleneck. Using `Arc<RwLock<HashMap<String, Regex>>>` to pre-compile and cache them transforms the cost to O(1) lookup + execution.
**Action:** Always look for `Regex::new` inside loops or request handlers. Use `lazy_static`, `OnceLock`, or struct-level caching to compile once.
