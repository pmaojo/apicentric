import { test as setup, expect } from '@playwright/test';
import { ApiTestHelper } from './utils/api-helper';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

setup('verify backend is running', async ({ request }) => {
  console.log('🔍 Checking if backend is available...');
  
  const apiHelper = new ApiTestHelper(BACKEND_URL);
  
  try {
    // Try to reach the health endpoint
    const response = await request.get(`${BACKEND_URL}/health`);
    expect(response.ok()).toBeTruthy();
    console.log('✅ Backend health check passed');
  } catch (error) {
    console.error('❌ Backend health check failed:', error);
    console.log('🚨 Make sure the backend is running on', BACKEND_URL);
    throw error;
  }

  try {
    // Check simulator status
    const status = await apiHelper.getSimulatorStatus();
    console.log('📊 Simulator status:', {
      isActive: status?.is_active,
      servicesCount: status?.services_count,
      activeServices: status?.active_services?.length || 0
    });
  } catch (error) {
    console.log('⚠️ Could not get simulator status:', error);
    // Don't fail setup for this, as simulator might not be started yet
  }
  
  console.log('✅ Backend setup verification completed');
});