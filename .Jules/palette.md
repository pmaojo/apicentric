## 2025-02-18 - Verifying Frontend without Backend
**Learning:** The WebUI renders a full-page `BackendConnectionError` if `fetchSimulatorStatus` fails. To verify UI changes in isolation (like tooltips), use Playwright's `page.route` to mock the `/status` endpoint with active services.
**Action:** When working on UI components that depend on backend status, always mock `/status` in Playwright verification scripts.
