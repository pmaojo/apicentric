## 2024-05-24 - DoS vulnerability via Clock Skew Panics
**Vulnerability:** Several parts of the application called `SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()`. If the system clock goes backwards or experiences time anomalies, `duration_since` returns an error, causing a panic and crashing the application/thread (Denial of Service).
**Learning:** `SystemTime` relies on the system clock and is not guaranteed to be monotonic. Any operations calculating durations between `SystemTime` instances must safely handle `SystemTimeError`.
**Prevention:** Use `.unwrap_or_default()` when calling `duration_since` on `SystemTime` to provide a safe zero duration fallback, or use `Instant` instead of `SystemTime` if a monotonic clock is required for purely internal timings.

## 2024-05-24 - DoS Vulnerability via Dynamic Type Extraction Panics
**Vulnerability:** The Rhai scripting engine dynamically evaluates expressions, but the application was blindly calling `unwrap()` on type conversion methods (e.g., `.as_int().unwrap()`) when extracting variable values. An unexpected type evaluated from a script could trigger a panic, causing a Denial of Service.
**Learning:** External or script-generated data should never be trusted to conform to expected types perfectly.
**Prevention:** Use safe fallbacks like `.unwrap_or_default()` when converting dynamic types, or handle the error explicitly.

## 2024-05-24 - DoS Vulnerability via Mutex Poisoning Panics
**Vulnerability:** The application used `unwrap()` when acquiring `Mutex` locks on the shared Rhai scripting engine and its cache (`self.engine.lock().unwrap()`). If another thread panics while holding the lock, subsequent lock attempts will return a `PoisonError`, causing `unwrap()` to panic and crash the thread (Denial of Service).
**Learning:** `Mutex` and `RwLock` in Rust can become "poisoned" if a thread panics while holding them. Blindly unwrapping lock results is a stability risk in production.
**Prevention:** Handle potential lock poisoning safely without panicking. Options include recovering the lock guard via `.unwrap_or_else(|e| e.into_inner())` or mapping the error to a safe runtime error (`.map_err(...)`) to fail gracefully.
