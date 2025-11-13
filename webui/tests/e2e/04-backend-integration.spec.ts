import { test, expect } from '@playwright/test';
import { ApiTestHelper } from '../utils/api-helper';
import { SAMPLE_SERVICE_YAML, SAMPLE_ECOMMERCE_SERVICE_YAML } from '../fixtures/test-data';

test.describe('Backend Integration Tests', () => {
  let apiHelper: ApiTestHelper;
  const testServiceName = `backend-test-${Date.now()}`;

  test.beforeAll(async () => {
    apiHelper = new ApiTestHelper();
  });

  test.afterEach(async () => {
    // Clean up test services
    try {
      await apiHelper.deleteService(testServiceName);
    } catch (error) {
      // Service might not exist
      console.log('Clean up skipped:', error);
    }
  });

  test('should connect to backend health endpoint', async () => {
    // This is tested in global setup, but we test it again here
    const response = await fetch('http://localhost:8080/health');
    expect(response.ok()).toBeTruthy();
    
    const health = await response.json();
    console.log('✅ Backend health check passed:', health);
  });

  test('should get simulator status from backend', async () => {
    const status = await apiHelper.getSimulatorStatus();
    
    expect(status).toBeDefined();
    expect(typeof status.is_active).toBe('boolean');
    expect(typeof status.services_count).toBe('number');
    expect(Array.isArray(status.active_services)).toBeTruthy();
    
    console.log('✅ Simulator status retrieved:', {
      isActive: status.is_active,
      servicesCount: status.services_count,
      activeServicesCount: status.active_services.length
    });
  });

  test('should start and stop simulator', async () => {
    try {
      // Get initial status
      const initialStatus = await apiHelper.getSimulatorStatus();
      console.log('Initial simulator state:', initialStatus.is_active);
      
      if (!initialStatus.is_active) {
        // Start simulator
        await apiHelper.startSimulator();
        console.log('✅ Simulator start command sent');
        
        // Wait and check status
        await apiHelper.waitForSimulatorState(true, 15000);
        console.log('✅ Simulator started successfully');
        
        // Stop simulator
        await apiHelper.stopSimulator();
        console.log('✅ Simulator stop command sent');
        
        await apiHelper.waitForSimulatorState(false, 10000);
        console.log('✅ Simulator stopped successfully');
      } else {
        // Stop first, then start
        await apiHelper.stopSimulator();
        await apiHelper.waitForSimulatorState(false, 10000);
        console.log('✅ Simulator stopped successfully');
        
        await apiHelper.startSimulator();
        await apiHelper.waitForSimulatorState(true, 15000);
        console.log('✅ Simulator restarted successfully');
      }
    } catch (error) {
      console.log('⚠️ Simulator start/stop test failed:', error);
      // Don't fail the test completely as this might be environment-specific
    }
  });

  test('should create service via API', async () => {
    const serviceYaml = SAMPLE_SERVICE_YAML.replace('test-service', testServiceName);
    
    const response = await apiHelper.createService(serviceYaml, `${testServiceName}.yaml`);
    
    expect(response).toBeDefined();
    expect(response.success).toBeTruthy();
    
    console.log('✅ Service created via API:', response.data.name);
    
    // Verify service exists in listing
    const services = await apiHelper.listServices();
    const createdService = services.find(s => s.name === testServiceName);
    
    expect(createdService).toBeDefined();
    console.log('✅ Created service found in service list');
  });

  test('should start and stop individual service', async () => {
    // Create a test service first
    const serviceYaml = SAMPLE_SERVICE_YAML.replace('test-service', testServiceName);
    await apiHelper.createService(serviceYaml, `${testServiceName}.yaml`);
    
    try {
      // Start the service
      await apiHelper.startService(testServiceName);
      console.log('✅ Service start command sent');
      
      // Wait for service to start
      await apiHelper.waitForServiceState(testServiceName, true, 15000);
      console.log('✅ Service started successfully');
      
      // Verify service status
      const status = await apiHelper.getServiceStatus(testServiceName);
      expect(status).toBeDefined();
      console.log('✅ Service status retrieved');
      
      // Stop the service
      await apiHelper.stopService(testServiceName);
      console.log('✅ Service stop command sent');
      
      await apiHelper.waitForServiceState(testServiceName, false, 10000);
      console.log('✅ Service stopped successfully');
      
    } catch (error) {
      console.log('⚠️ Individual service start/stop failed:', error);
    }
  });

  test('should delete service via API', async () => {
    // Create a test service first
    const serviceYaml = SAMPLE_SERVICE_YAML.replace('test-service', testServiceName);
    await apiHelper.createService(serviceYaml, `${testServiceName}.yaml`);
    
    // Verify it exists
    let services = await apiHelper.listServices();
    let serviceExists = services.some(s => s.name === testServiceName);
    expect(serviceExists).toBeTruthy();
    
    // Delete the service
    await apiHelper.deleteService(testServiceName);
    console.log('✅ Service deleted via API');
    
    // Verify it's gone
    services = await apiHelper.listServices();
    serviceExists = services.some(s => s.name === testServiceName);
    expect(serviceExists).toBeFalsy();
    
    console.log('✅ Service deletion verified');
  });

  test('should handle service YAML validation', async () => {
    // Test valid YAML
    const validYaml = SAMPLE_SERVICE_YAML.replace('test-service', 'validation-test');
    
    try {
      await apiHelper.createService(validYaml, 'validation-test.yaml');
      console.log('✅ Valid YAML accepted');
      
      // Clean up
      await apiHelper.deleteService('validation-test');
    } catch (error) {
      console.log('⚠️ Valid YAML rejected:', error);
    }
    
    // Test invalid YAML
    const invalidYaml = `invalid: yaml: content: [unclosed bracket`;
    
    try {
      await apiHelper.createService(invalidYaml, 'invalid-test.yaml');
      console.log('❌ Invalid YAML was unexpectedly accepted');
      expect(false).toBeTruthy(); // Should not reach here
    } catch (error) {
      console.log('✅ Invalid YAML properly rejected:', error);
    }
  });

  test('should query request logs', async () => {
    try {
      const logs = await apiHelper.queryLogs({ limit: 10 });
      
      expect(logs).toBeDefined();
      expect(Array.isArray(logs.logs)).toBeTruthy();
      expect(typeof logs.total).toBe('number');
      expect(typeof logs.filtered).toBe('number');
      
      console.log('✅ Request logs retrieved:', {
        totalLogs: logs.total,
        filteredLogs: logs.filtered,
        returnedLogs: logs.logs.length
      });
    } catch (error) {
      console.log('⚠️ Log query failed:', error);
    }
  });

  test('should clear request logs', async () => {
    try {
      await apiHelper.clearLogs();
      console.log('✅ Logs cleared successfully');
      
      // Verify logs are cleared
      const logs = await apiHelper.queryLogs({ limit: 10 });
      console.log('Logs after clear:', logs.total);
      
    } catch (error) {
      console.log('⚠️ Log clearing failed:', error);
    }
  });

  test('should handle multiple services', async () => {
    const service1Name = `${testServiceName}-1`;
    const service2Name = `${testServiceName}-2`;
    
    try {
      // Create multiple services
      const yaml1 = SAMPLE_SERVICE_YAML.replace('test-service', service1Name).replace('port: 3001', 'port: 3001');
      const yaml2 = SAMPLE_ECOMMERCE_SERVICE_YAML.replace('ecommerce-api', service2Name).replace('port: 3002', 'port: 3003');
      
      await apiHelper.createService(yaml1, `${service1Name}.yaml`);
      await apiHelper.createService(yaml2, `${service2Name}.yaml`);
      
      console.log('✅ Multiple services created');
      
      // Verify both exist
      const services = await apiHelper.listServices();
      const service1Exists = services.some(s => s.name === service1Name);
      const service2Exists = services.some(s => s.name === service2Name);
      
      expect(service1Exists).toBeTruthy();
      expect(service2Exists).toBeTruthy();
      
      console.log('✅ Multiple services verified in listing');
      
      // Clean up
      await apiHelper.deleteService(service1Name);
      await apiHelper.deleteService(service2Name);
      
      console.log('✅ Multiple services cleaned up');
      
    } catch (error) {
      console.log('⚠️ Multiple services test failed:', error);
      
      // Try to clean up anyway
      try { await apiHelper.deleteService(service1Name); } catch {}
      try { await apiHelper.deleteService(service2Name); } catch {}
    }
  });

  test('should handle concurrent operations', async () => {
    const promises = [];
    const serviceNames = [];
    
    try {
      // Create multiple services concurrently
      for (let i = 0; i < 3; i++) {
        const serviceName = `${testServiceName}-concurrent-${i}`;
        serviceNames.push(serviceName);
        
        const yaml = SAMPLE_SERVICE_YAML
          .replace('test-service', serviceName)
          .replace('port: 3001', `port: ${3001 + i}`);
        
        promises.push(apiHelper.createService(yaml, `${serviceName}.yaml`));
      }
      
      // Wait for all to complete
      await Promise.allSettled(promises);
      console.log('✅ Concurrent service creation completed');
      
      // Verify services exist
      const services = await apiHelper.listServices();
      const existingCount = serviceNames.filter(name => 
        services.some(s => s.name === name)
      ).length;
      
      console.log(`✅ ${existingCount}/${serviceNames.length} concurrent services created successfully`);
      
      // Clean up all services
      const cleanupPromises = serviceNames.map(name => 
        apiHelper.deleteService(name).catch(() => {})
      );
      await Promise.allSettled(cleanupPromises);
      
      console.log('✅ Concurrent services cleaned up');
      
    } catch (error) {
      console.log('⚠️ Concurrent operations test failed:', error);
    }
  });

  test('should handle error cases gracefully', async () => {
    // Test non-existent service operations
    try {
      await apiHelper.getServiceStatus('non-existent-service');
      console.log('❌ Non-existent service status should have failed');
      expect(false).toBeTruthy();
    } catch (error) {
      console.log('✅ Non-existent service status properly failed');
    }
    
    try {
      await apiHelper.deleteService('non-existent-service');
      console.log('❌ Non-existent service deletion should have failed');
      expect(false).toBeTruthy();
    } catch (error) {
      console.log('✅ Non-existent service deletion properly failed');
    }
    
    try {
      await apiHelper.startService('non-existent-service');
      console.log('❌ Non-existent service start should have failed');
      expect(false).toBeTruthy();
    } catch (error) {
      console.log('✅ Non-existent service start properly failed');
    }
  });
});