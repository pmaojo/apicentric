import { test, expect } from '@playwright/test';
import { WebUIHelper } from '../utils/webui-helper';
import { ApiTestHelper } from '../utils/api-helper';
import { EXPECTED_UI_ELEMENTS } from '../fixtures/test-data';

test.describe('Basic E2E Navigation Tests', () => {
  let webUI: WebUIHelper;
  let apiHelper: ApiTestHelper;

  test.beforeEach(async ({ page }) => {
    webUI = new WebUIHelper(page);
    apiHelper = new ApiTestHelper();
    await webUI.navigateToHome();
  });

  test('should load the application successfully', async ({ page }) => {
    // Check that the main layout loads
    await expect(page).toHaveTitle(/Apicentric/);
    
    // Check that the sidebar is visible
    await expect(page.getByText('Apicentric', { exact: true })).toBeVisible();
    
    // Check that we're on the dashboard by default
    await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  });

  test('should navigate through all main views', async ({ page }) => {
    // Test navigation to each main view
    for (const view of EXPECTED_UI_ELEMENTS.views) {
      const viewId = view.toLowerCase().replace(/ /g, '-').replace('client-', '');
      
      if (viewId === 'dashboard') {
        await webUI.navigateToDashboard();
      } else if (viewId === 'service-definitions') {
        await webUI.navigateToServices();
      } else if (viewId === 'simulator-logs') {
        await webUI.navigateToLogs();
      } else if (viewId === 'ai-service-generator') {
        await webUI.navigateToAIGenerator();
      } else if (viewId === 'code-generator') {
        await webUI.navigateToCodeGenerator();
      } else if (viewId === 'recording') {
        await webUI.navigateToRecording();
      } else if (viewId === 'configuration') {
        await webUI.navigateToConfiguration();
      }
      
      // Verify the view loaded
      await expect(page.getByRole('heading', { name: view })).toBeVisible();
      
      // Take a screenshot for debugging
      await webUI.takeScreenshot(`view-${viewId}`);
    }
  });

  test('should have all sidebar navigation items', async ({ page }) => {
    // Check that all expected sidebar items are present
    for (const sidebarItem of EXPECTED_UI_ELEMENTS.sidebar) {
      const sidebarElement = page.getByTestId(`sidebar-${sidebarItem}`);
      await expect(sidebarElement).toBeVisible();
    }
  });

  test('should display simulator status card', async ({ page }) => {
    // Check that simulator status card is present
    await expect(page.getByText('Simulator Status')).toBeVisible();
    
    // Check for reload button
    const reloadButton = await webUI.getReloadAllButton();
    await expect(reloadButton).toBeVisible();
    
    // Check for status text
    const statusText = await webUI.getSimulatorStatusText();
    expect(statusText).toMatch(/Simulator is (running|stopped)/);
  });

  test('should navigate back to dashboard from any view', async ({ page }) => {
    // Navigate to different views and always return to dashboard
    const testViews = ['services', 'logs', 'ai-generator'];
    
    for (const view of testViews) {
      // Navigate to a view
      await webUI.clickSidebarItem(view);
      
      // Navigate back to dashboard
      await webUI.navigateToDashboard();
      await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
    }
  });

  test('should maintain responsive behavior on mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 390, height: 844 });
    
    // Navigate to home
    await webUI.navigateToHome();
    
    // Check that the layout is responsive
    await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Toggle Sidebar' })).toBeVisible();
    
    // Try navigation on mobile
    await page.getByRole('button', { name: 'Toggle Sidebar' }).click();
    await webUI.navigateToServices();
    await expect(page.getByRole('heading', { name: 'Service Definitions' })).toBeVisible();
  });

  test('should handle page refresh correctly', async ({ page }) => {
    // Navigate to a specific view
    await webUI.navigateToServices();
    await expect(page.getByRole('heading', { name: 'Service Definitions' })).toBeVisible();
    
    // Refresh the page
    await page.reload();
    
    // Check that we're still on the services page (or back to dashboard)
    // The app might reset to dashboard after refresh, which is acceptable
    const title = await webUI.getPageTitle();
    expect(title).toContain('Apicentric');
  });

  test('should handle network errors gracefully', async ({ page }) => {
    // Navigate to dashboard
    await webUI.navigateToDashboard();
    
    // Block network requests to simulate offline state
    await page.route('**/status', route => route.abort());
    await page.route('**/api/**', route => route.abort());
    
    // Try to navigate - app should still function for basic navigation
    await webUI.navigateToServices();
    await expect(page.getByRole('heading', { name: 'Service Definitions' })).toBeVisible();
    
    // App should show loading states or error states, but not crash
    // We don't test specific error messages here as they may vary
  });
});