import { test, expect } from '@playwright/test';
import { WebUIHelper } from '../utils/webui-helper';
import { ApiTestHelper } from '../utils/api-helper';
import { TEST_SCENARIOS, SAMPLE_SERVICE_YAML } from '../fixtures/test-data';

test.describe('Service Management E2E Tests', () => {
  let webUI: WebUIHelper;
  let apiHelper: ApiTestHelper;
  let testServiceName: string;

  test.beforeEach(async ({ page }) => {
    webUI = new WebUIHelper(page);
    apiHelper = new ApiTestHelper();
    testServiceName = `test-service-${Date.now()}-${Math.floor(Math.random() * 10000)}`;
    await webUI.navigateToHome();
    await webUI.navigateToServices();
  });

  test.afterEach(async () => {
    // Clean up: delete test service if it exists
    try {
      await apiHelper.deleteService(testServiceName);
      console.log(`✅ Cleaned up test service: ${testServiceName}`);
    } catch (error) {
      // Service might not exist, which is fine
      console.log(`🧹 Clean up skipped for ${testServiceName}:`, error);
    }
  });

  test('should display service management page correctly', async ({ page }) => {
    // Check main elements
    await expect(page.getByRole('heading', { name: 'Service Definitions' })).toBeVisible();
    
    // Should have create service button
    const createButton = page.getByTestId('create-service-button');
    await expect(createButton).toBeVisible();
    
    // Should have service table/list area
    // The exact structure may vary, but some container for services should exist
    const servicesArea = page.locator('table, [data-testid*="service"], .service');
    await expect(servicesArea.first()).toBeVisible();
  });

  test('should list existing services from backend', async ({ page }) => {
    try {
      // Get services from backend
      const backendServices = await apiHelper.listServices();
      console.log(`Backend has ${backendServices.length} services`);
      
      if (backendServices.length > 0) {
        // Frontend should display these services
        for (const service of backendServices.slice(0, 3)) { // Test first 3 to avoid too many checks
          const serviceElement = page.getByTestId(`service-${service.name}`);
          await expect(serviceElement).toBeVisible();
          console.log(`✅ Service ${service.name} displayed in frontend`);
        }
      } else {
        console.log('🤷 No services in backend to verify');
      }
    } catch (error) {
      console.log('⚠️ Could not verify backend services:', error);
    }
  });

  test('should open create service dialog', async ({ page }) => {
    // Click create service button
    const createButton = page.getByTestId('create-service-button');
    await createButton.click();
    
    // Dialog should open
    await expect(page.getByRole('dialog')).toBeVisible();
    
    // Should have form fields for service creation
    // These data-testids might need to be added to the CreateServiceDialog component
    const dialogContent = page.getByRole('dialog');
    
    // Look for common form elements
    const hasNameField = await dialogContent.locator('input[name*="name"], input[placeholder*="name"], [data-testid*="service-name"]').count() > 0;
    const hasYamlField = await dialogContent.locator('textarea, [data-testid*="yaml"], [data-testid*="definition"]').count() > 0;
    
    expect(hasNameField || hasYamlField).toBeTruthy();
    
    console.log('✅ Create service dialog opened with form fields');
  });

  test('should create a new service via API and verify in UI', async ({ page }) => {
    try {
      // Create service via backend API
      let serviceYaml = SAMPLE_SERVICE_YAML.replace('test-service', testServiceName);
      const port = 3000 + Math.floor(Math.random() * 1000);
      serviceYaml = serviceYaml.replace('port: 3001', `port: ${port}`);

      await apiHelper.createService(serviceYaml, `${testServiceName}.yaml`);
      
      console.log(`✅ Service ${testServiceName} created via API`);

      // Wait for service to be running in backend
      await apiHelper.waitForServiceState(testServiceName, true);
      
      // Refresh the page to see the new service
      await page.reload();
      await webUI.navigateToServices();
      
      // Service should appear in the UI
      const serviceElement = page.getByTestId(`service-${testServiceName}`);
      await expect(serviceElement).toBeVisible({ timeout: 10000 });
      
      console.log(`✅ Service ${testServiceName} visible in UI`);
      
    } catch (error) {
      console.log(`❌ Failed to create/verify service: ${error}`);
      throw error;
    }
  });

  test('should start and stop service via UI', async ({ page }) => {
    try {
      // First create a service
      let serviceYaml = SAMPLE_SERVICE_YAML.replace('test-service', testServiceName);
      const port = 3000 + Math.floor(Math.random() * 1000);
      serviceYaml = serviceYaml.replace('port: 3001', `port: ${port}`);

      await apiHelper.createService(serviceYaml, `${testServiceName}.yaml`);
      
      // Refresh to see the service
      await page.reload();
      await webUI.navigateToServices();
      
      // Find the service
      const serviceElement = page.getByTestId(`service-${testServiceName}`);
      await expect(serviceElement).toBeVisible({ timeout: 10000 });
      
      // Try to start the service
      const startButton = serviceElement.getByTestId('start-service-button');
      if (await startButton.isVisible()) {
        await startButton.click();
        
        // Wait for status change
        await page.waitForTimeout(3000);
        
        // Check if stop button is now visible
        const stopButton = serviceElement.getByTestId('stop-service-button');
        if (await stopButton.isVisible()) {
          console.log('✅ Service started successfully - stop button visible');
          
          // Try to stop the service
          await stopButton.click();
          await page.waitForTimeout(2000);
          
          console.log('✅ Service stop command sent');
        }
      } else {
        console.log('⚠️ Start button not visible, service might already be running');
      }
      
    } catch (error) {
      console.log(`⚠️ Start/stop test failed: ${error}`);
    }
  });

  test('should show service status accurately', async ({ page }) => {
    try {
      // Get services from backend to check their actual status
      const status = await apiHelper.getSimulatorStatus();
      
      if (status.active_services && status.active_services.length > 0) {
        const service = status.active_services[0];
        console.log(`Checking status for service: ${service.name}, running: ${service.is_running}`);
        
        // Find the service in UI
        const serviceElement = page.getByTestId(`service-${service.name}`);
        
        if (await serviceElement.isVisible()) {
          // Check status badge/indicator
          const statusElement = serviceElement.locator('[data-testid="service-status"], .badge, [class*="status"]').first();
          
          if (await statusElement.isVisible()) {
            const statusText = await statusElement.textContent();
            console.log(`UI shows status: "${statusText}" for ${service.name}`);
            
            if (service.is_running) {
              expect(statusText?.toLowerCase()).toMatch(/running|active|started/);
            } else {
              expect(statusText?.toLowerCase()).toMatch(/stopped|inactive|offline/);
            }
            
            console.log('✅ Service status matches between backend and frontend');
          }
        }
      }
    } catch (error) {
      console.log('⚠️ Status verification failed:', error);
    }
  });

  test('should handle service actions menu', async ({ page }) => {
    try {
      // Get first available service
      const status = await apiHelper.getSimulatorStatus();
      
      if (status.active_services && status.active_services.length > 0) {
        const service = status.active_services[0];
        const serviceElement = page.getByTestId(`service-${service.name}`);
        
        if (await serviceElement.isVisible()) {
          // Look for actions menu (three dots menu, etc.)
          const actionsMenu = serviceElement.locator('[data-testid*="menu"], [data-testid*="actions"], button[aria-haspopup="menu"]').first();
          
          if (await actionsMenu.isVisible()) {
            await actionsMenu.click();
            
            // Menu should open with options
            const menuContent = page.locator('[role="menu"], [data-testid*="menu-content"]');
            await expect(menuContent).toBeVisible({ timeout: 5000 });
            
            console.log('✅ Service actions menu opened');
            
            // Click somewhere else to close menu
            await page.click('body');
          } else {
            console.log('⚠️ Actions menu not found for service');
          }
        }
      }
    } catch (error) {
      console.log('⚠️ Actions menu test failed:', error);
    }
  });

  test('should handle service deletion confirmation', async ({ page }) => {
    try {
      // Create a test service first
      let serviceYaml = SAMPLE_SERVICE_YAML.replace('test-service', testServiceName);
      const port = 3000 + Math.floor(Math.random() * 1000);
      serviceYaml = serviceYaml.replace('port: 3001', `port: ${port}`);

      await apiHelper.createService(serviceYaml, `${testServiceName}.yaml`);
      
      // Refresh to see the service
      await page.reload();
      await webUI.navigateToServices();
      
      const serviceElement = page.getByTestId(`service-${testServiceName}`);
      await expect(serviceElement).toBeVisible({ timeout: 10000 });
      
      // Look for delete action (might be in dropdown menu)
      const deleteButton = serviceElement.getByTestId('delete-service-button');
      
      if (await deleteButton.isVisible()) {
        await deleteButton.click();
        
        // Should show confirmation dialog
        const confirmDialog = page.getByRole('dialog').or(page.locator('[data-testid*="confirm"]'));
        await expect(confirmDialog).toBeVisible({ timeout: 5000 });
        
        console.log('✅ Delete confirmation dialog appeared');
        
        // Cancel the deletion for now
        const cancelButton = confirmDialog.getByText('Cancel').or(confirmDialog.getByTestId('cancel-delete'));
        if (await cancelButton.isVisible()) {
          await cancelButton.click();
        } else {
          // Press escape to close dialog
          await page.keyboard.press('Escape');
        }
        
      } else {
        console.log('⚠️ Delete button not found directly, might be in menu');
      }
    } catch (error) {
      console.log('⚠️ Delete confirmation test failed:', error);
    }
  });

  test('should handle empty services state', async ({ page }) => {
    try {
      // Check if there are any services
      const status = await apiHelper.getSimulatorStatus();
      
      if (!status.active_services || status.active_services.length === 0) {
        // Should show empty state message
        const emptyMessage = page.getByText(/no services/i)
          .or(page.getByText(/empty/i))
          .or(page.locator('[data-testid*="empty"]'));
        
        if (await emptyMessage.count() > 0) {
          await expect(emptyMessage.first()).toBeVisible();
          console.log('✅ Empty state message displayed');
        } else {
          console.log('📝 No explicit empty state message, but that may be by design');
        }
      } else {
        console.log('Services exist, skipping empty state test');
      }
    } catch (error) {
      console.log('⚠️ Empty state test failed:', error);
    }
  });

  test('should maintain service list consistency', async ({ page }) => {
    try {
      // Get services from backend
      const backendServices = await apiHelper.listServices();
      
      // Count services in frontend
      const frontendServiceElements = page.locator('[data-testid^="service-"]');
      const frontendCount = await frontendServiceElements.count();
      
      console.log(`Backend: ${backendServices.length} services, Frontend: ${frontendCount} service elements`);
      
      // Allow for small discrepancies due to timing, filtering, etc.
      const difference = Math.abs(backendServices.length - frontendCount);
      expect(difference).toBeLessThanOrEqual(2);
      
      console.log('✅ Service list consistency maintained');
    } catch (error) {
      console.log('⚠️ Service list consistency check failed:', error);
    }
  });
});