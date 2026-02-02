from playwright.sync_api import sync_playwright

def verify_dashboard():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()

        # Mock the status endpoint to simulate a running backend
        page.route("**/status", lambda route: route.fulfill(
            status=200,
            content_type="application/json",
            body='''{
                "data": {
                    "is_active": true,
                    "services_count": 2,
                    "active_services": [
                        {
                            "name": "User Service",
                            "version": "1.0.0",
                            "port": 3001,
                            "is_running": true,
                            "endpoints": [
                                {"method": "GET", "path": "/users"},
                                {"method": "POST", "path": "/users"}
                            ],
                            "definition": "openapi: 3.0.0\\ninfo:\\n  title: User Service\\n  version: 1.0.0"
                        },
                        {
                            "name": "Order Service",
                            "version": "2.1.0",
                            "port": 3002,
                            "is_running": false,
                            "endpoints": [
                                {"method": "GET", "path": "/orders"}
                            ],
                            "definition": "openapi: 3.0.0\\ninfo:\\n  title: Order Service\\n  version: 2.1.0"
                        }
                    ]
                }
            }'''
        ))

        # Also mock system metrics to avoid errors in SystemMetrics component
        page.route("**/api/system/metrics", lambda route: route.fulfill(
             status=200,
             content_type="application/json",
             body='{"data": {"cpu": 10, "memory": 20, "uptime": 100}}'
        ))

        # Mock logs for websocket or other polls
        page.route("**/api/logs*", lambda route: route.fulfill(
             status=200,
             content_type="application/json",
             body='{"data": {"logs": [], "total": 0, "filtered": 0}}'
        ))

        print("Navigating to dashboard...")
        page.goto("http://localhost:9002")

        # Wait for the dashboard to load (look for "Simulator Status")
        print("Waiting for dashboard content...")
        try:
            page.wait_for_selector("text=Simulator Status", timeout=10000)
            print("Dashboard loaded.")
        except:
            print("Timed out waiting for 'Simulator Status'. Taking screenshot anyway.")

        # Check for service cards
        print("Checking for service cards...")
        if page.locator("text=User Service").is_visible():
            print("User Service card is visible.")
        else:
            print("User Service card NOT visible.")

        # Take screenshot
        output_path = "verification/dashboard.png"
        page.screenshot(path=output_path, full_page=True)
        print(f"Screenshot saved to {output_path}")

        browser.close()

if __name__ == "__main__":
    verify_dashboard()
