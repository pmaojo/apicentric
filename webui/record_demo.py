import asyncio
from playwright.async_api import async_playwright
import json
import os

async def run():
    async with async_playwright() as p:
        # Launch browser with video recording
        browser = await p.chromium.launch(
            headless=True,
            args=['--no-sandbox', '--disable-setuid-sandbox']
        )

        video_dir = "webui"
        if not os.path.exists(video_dir):
            os.makedirs(video_dir)

        context = await browser.new_context(
            record_video_dir=video_dir,
            record_video_size={"width": 1280, "height": 720},
            viewport={"width": 1280, "height": 720}
        )

        page = await context.new_page()

        # Mock API responses

        # 1. Simulator Status
        async def handle_status(route):
            await route.fulfill(
                status=200,
                content_type="application/json",
                body=json.dumps({
                    "success": True,
                    "data": {
                        "is_active": True,
                        "services_count": 2,
                        "active_services": [
                            {
                                "id": "service-1",
                                "name": "users-service",
                                "version": "1.0.0",
                                "port": 3001,
                                "is_running": True,
                                "endpoints": [
                                    { "method": "GET", "path": "/users", "description": "Get all users" },
                                    { "method": "POST", "path": "/users", "description": "Create user" }
                                ],
                                "endpoints_count": 2,
                                "definition": "name: users-service\nversion: 1.0.0"
                            },
                            {
                                "id": "service-2",
                                "name": "payment-service",
                                "version": "1.2.0",
                                "port": 3002,
                                "is_running": False,
                                "endpoints": [
                                    { "method": "POST", "path": "/pay", "description": "Process payment" }
                                ],
                                "endpoints_count": 1,
                                "definition": "name: payment-service\nversion: 1.2.0"
                            }
                        ]
                    }
                })
            )
        await page.route("**/status", handle_status)
        await page.route("**/api/services/reload", lambda route: route.fulfill(status=200, body=json.dumps({"success": True})))

        # 2. Services List
        async def handle_services(route):
            if route.request.method == "GET":
                await route.fulfill(
                    status=200,
                    content_type="application/json",
                    body=json.dumps({
                        "success": True,
                        "data": [
                            {
                                "name": "users-service",
                                "version": "1.0.0",
                                "port": 3001,
                                "is_running": True,
                                "endpoints_count": 2
                            },
                            {
                                "name": "payment-service",
                                "version": "1.2.0",
                                "port": 3002,
                                "is_running": False,
                                "endpoints_count": 1
                            }
                        ]
                    })
                )
            else:
                await route.continue_()
        await page.route("**/api/services", handle_services)

        # 3. Logs
        async def handle_logs(route):
            await route.fulfill(
                status=200,
                content_type="application/json",
                body=json.dumps({
                    "success": True,
                    "data": {
                        "logs": [
                            {
                                "timestamp": "2023-10-27T10:00:00Z",
                                "service": "users-service",
                                "method": "GET",
                                "path": "/users",
                                "status": 200,
                                "duration_ms": 15
                            },
                            {
                                "timestamp": "2023-10-27T10:00:05Z",
                                "service": "payment-service",
                                "method": "POST",
                                "path": "/pay",
                                "status": 500,
                                "duration_ms": 45
                            }
                        ],
                        "total": 2,
                        "filtered": 2
                    }
                })
            )
        await page.route("**/api/logs**", handle_logs)

        # 4. IoT Twins
        async def handle_twins(route):
            await route.fulfill(
                status=200,
                content_type="application/json",
                body=json.dumps({
                    "success": True,
                    "data": ["thermostat-twin", "vehicle-twin"]
                })
            )
        await page.route("**/api/iot/twins", handle_twins)

        # 5. Marketplace
        async def handle_marketplace(route):
            await route.fulfill(
                status=200,
                content_type="application/json",
                body=json.dumps({
                    "success": True,
                    "data": [
                        {
                            "id": "1",
                            "name": "Auth Service",
                            "description": "Standard authentication service",
                            "category": "Security",
                            "definition_url": "http://example.com/auth.yaml"
                        }
                    ]
                })
            )
        await page.route("**/api/marketplace", handle_marketplace)

        # 6. AI Config
        await page.route("**/api/ai/config", lambda route: route.fulfill(
            status=200,
            body=json.dumps({
                "success": True,
                "data": {
                    "is_configured": True,
                    "provider": "openai",
                    "issues": []
                }
            })
        ))

        # 7. Generic successful response for other POST/PUTs to avoid errors
        async def handle_generic(route):
            if route.request.method in ["POST", "PUT", "DELETE"]:
                 await route.fulfill(
                    status=200,
                    content_type="application/json",
                    body=json.dumps({"success": True, "data": {}})
                )
            else:
                await route.continue_()

        # Capture remaining API calls
        await page.route("**/api/**", handle_generic)

        # Start Navigation
        print("Navigating to Dashboard...")
        try:
            await page.goto("http://localhost:9002")
            await page.wait_for_load_state("networkidle")
        except Exception as e:
            print(f"Error navigating: {e}")
            await context.close()
            return

        # 1. Dashboard Interaction
        print("Interacting with Dashboard...")
        try:
            await page.wait_for_selector("text=Simulator Status", timeout=10000)
            await page.wait_for_timeout(2000) # Let animations settle

            # Hover over a service card
            await page.hover("[data-testid='service-card'][data-service-name='users-service']")
            await page.wait_for_timeout(1000)

            # 2. Services
            print("Navigating to Services...")
            await page.click("[data-testid='sidebar-services']")
            await page.wait_for_timeout(2000)

            # Try to find Add Service button - it might be an icon or text
            # Looking at dashboard.tsx (which isn't services list), I need to check ServiceManagement component
            # But I am in Dashboard.

            # Wait for content
            await page.wait_for_timeout(1000)

            # 3. IoT
            print("Navigating to IoT...")
            await page.click("[data-testid='sidebar-iot']")
            await page.wait_for_timeout(2000)

            # 4. Marketplace
            print("Navigating to Marketplace...")
            await page.click("[data-testid='sidebar-marketplace']")
            await page.wait_for_timeout(2000)

            # 5. Recording
            print("Navigating to Recording...")
            await page.click("[data-testid='sidebar-recording']")
            await page.wait_for_timeout(2000)

            # 6. AI Generator
            print("Navigating to AI Generator...")
            await page.click("[data-testid='sidebar-ai-generator']")
            await page.wait_for_timeout(2000)

            # 7. Plugin Generator
            print("Navigating to Plugin Generator...")
            await page.click("[data-testid='sidebar-plugin-generator']")
            await page.wait_for_timeout(2000)

            # 8. Contract Testing
            print("Navigating to Contract Testing...")
            await page.click("[data-testid='sidebar-contract-testing']")
            await page.wait_for_timeout(2000)

            # 9. Code Generator
            print("Navigating to Code Generator...")
            await page.click("[data-testid='sidebar-code-generator']")
            await page.wait_for_timeout(2000)

            # 10. Logs
            print("Navigating to Logs...")
            await page.click("[data-testid='sidebar-logs']")
            await page.wait_for_timeout(2000)

            # 11. Configuration
            print("Navigating to Configuration...")
            await page.click("[data-testid='sidebar-configuration']")
            await page.wait_for_timeout(2000)

            # Back to Dashboard
            print("Back to Dashboard...")
            await page.click("[data-testid='sidebar-dashboard']")
            await page.wait_for_timeout(2000)

        except Exception as e:
            print(f"Error during interaction: {e}")
            # Take screenshot if something fails
            await page.screenshot(path="webui/error.png")

        await context.close()
        await browser.close()

        # Rename video file
        files = os.listdir(video_dir)
        renamed = False
        for f in files:
             if f.endswith(".webm") and not f == "demo_video.webm":
                 old_path = os.path.join(video_dir, f)
                 new_path = os.path.join(video_dir, "demo_video.webm")
                 if os.path.exists(new_path):
                     os.remove(new_path)
                 os.rename(old_path, new_path)
                 print(f"Renamed {f} to demo_video.webm")
                 renamed = True
                 break

        if not renamed and os.path.exists(os.path.join(video_dir, "demo_video.webm")):
            print("demo_video.webm already exists (updated).")

if __name__ == "__main__":
    asyncio.run(run())
