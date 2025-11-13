import { Page, Locator, expect } from '@playwright/test';

/**
 * Page Object Model for common frontend interactions
 */
export class WebUIHelper {
  constructor(private page: Page) {}

  // Navigation helpers
  async navigateToHome() {
    await this.page.goto('/');
    await this.waitForLoad();
  }

  async navigateToDashboard() {
    await this.clickSidebarItem('dashboard');
    await this.waitForViewLoad('Dashboard');
  }

  async navigateToServices() {
    await this.clickSidebarItem('services');
    await this.waitForViewLoad('Service Definitions');
  }

  async navigateToLogs() {
    await this.clickSidebarItem('logs');
    await this.waitForViewLoad('Simulator Logs');
  }

  async navigateToAIGenerator() {
    await this.clickSidebarItem('ai-generator');
    await this.waitForViewLoad('AI Service Generator');
  }

  async navigateToCodeGenerator() {
    await this.clickSidebarItem('code-generator');
    await this.waitForViewLoad('Client Code Generator');
  }

  async navigateToRecording() {
    await this.clickSidebarItem('recording');
    await this.waitForViewLoad('Recording');
  }

  async navigateToConfiguration() {
    await this.clickSidebarItem('configuration');
    await this.waitForViewLoad('Configuration');
  }

  // Sidebar interaction
  async clickSidebarItem(view: string) {
    const sidebarItem = this.page.getByTestId(`sidebar-${view}`);
    await expect(sidebarItem).toBeVisible();
    await sidebarItem.click();
  }

  // Wait helpers
  async waitForLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async waitForViewLoad(viewTitle: string) {
    await expect(this.page.getByRole('heading', { name: viewTitle })).toBeVisible();
  }

  // Simulator controls
  async getSimulatorToggleButton(): Promise<Locator> {
    return this.page.getByTestId('simulator-toggle');
  }

  async clickSimulatorToggle() {
    const toggle = await this.getSimulatorToggleButton();
    await expect(toggle).toBeVisible();
    await toggle.click();
  }

  async waitForSimulatorStatus(isRunning: boolean, timeoutMs: number = 10000) {
    const expectedText = isRunning ? 'Stop Simulator' : 'Start Simulator';
    await expect(this.page.getByTestId('simulator-toggle')).toContainText(expectedText, { timeout: timeoutMs });
  }

  // Service management
  async createService(name: string, yaml: string) {
    // Navigate to services if not already there
    await this.navigateToServices();
    
    // Click create service button
    const createButton = this.page.getByTestId('create-service-button');
    await expect(createButton).toBeVisible();
    await createButton.click();
    
    // Fill in the form
    await this.page.getByTestId('service-name-input').fill(name);
    await this.page.getByTestId('service-yaml-textarea').fill(yaml);
    
    // Submit
    await this.page.getByTestId('create-service-submit').click();
    
    // Wait for service to appear
    await expect(this.page.getByTestId(`service-${name}`)).toBeVisible();
  }

  async deleteService(name: string) {
    const serviceElement = this.page.getByTestId(`service-${name}`);
    await expect(serviceElement).toBeVisible();
    
    // Click delete button
    const deleteButton = serviceElement.getByTestId('delete-service-button');
    await deleteButton.click();
    
    // Confirm deletion
    await this.page.getByTestId('confirm-delete-button').click();
    
    // Wait for service to disappear
    await expect(serviceElement).not.toBeVisible();
  }

  async startService(name: string) {
    const serviceElement = this.page.getByTestId(`service-${name}`);
    await expect(serviceElement).toBeVisible();
    
    const startButton = serviceElement.getByTestId('start-service-button');
    await startButton.click();
    
    // Wait for status to change
    await expect(serviceElement.getByTestId('service-status')).toContainText('running');
  }

  async stopService(name: string) {
    const serviceElement = this.page.getByTestId(`service-${name}`);
    await expect(serviceElement).toBeVisible();
    
    const stopButton = serviceElement.getByTestId('stop-service-button');
    await stopButton.click();
    
    // Wait for status to change
    await expect(serviceElement.getByTestId('service-status')).toContainText('stopped');
  }

  // Dashboard helpers
  async getServiceCardsCount(): Promise<number> {
    const cards = this.page.getByTestId('service-card');
    return await cards.count();
  }

  async getRunningServicesCount(): Promise<number> {
    const runningCards = this.page.getByTestId('service-card').filter({ hasText: 'running' });
    return await runningCards.count();
  }

  // Logs helpers
  async clearLogs() {
    await this.navigateToLogs();
    const clearButton = this.page.getByTestId('clear-logs-button');
    await expect(clearButton).toBeVisible();
    await clearButton.click();
    
    // Confirm if needed
    try {
      await this.page.getByTestId('confirm-clear-logs').click({ timeout: 2000 });
    } catch {
      // No confirmation dialog appeared
    }
  }

  async filterLogsByService(serviceName: string) {
    await this.navigateToLogs();
    const serviceFilter = this.page.getByTestId('logs-service-filter');
    await serviceFilter.selectOption(serviceName);
  }

  async getLogEntriesCount(): Promise<number> {
    const entries = this.page.getByTestId('log-entry');
    return await entries.count();
  }

  // AI Generator helpers
  async generateServiceWithAI(prompt: string) {
    await this.navigateToAIGenerator();
    
    await this.page.getByTestId('ai-prompt-textarea').fill(prompt);
    await this.page.getByTestId('generate-ai-service-button').click();
    
    // Wait for generation to complete
    await expect(this.page.getByTestId('ai-generated-yaml')).toBeVisible();
    
    // Add the generated service
    await this.page.getByTestId('add-generated-service-button').click();
  }

  // Code Generator helpers
  async generateTypeScriptCode(serviceName: string): Promise<string> {
    await this.navigateToCodeGenerator();
    
    await this.page.getByTestId('service-select').selectOption(serviceName);
    await this.page.getByTestId('generate-typescript-button').click();
    
    await expect(this.page.getByTestId('generated-code')).toBeVisible();
    
    return await this.page.getByTestId('generated-code').textContent() || '';
  }

  // Recording helpers
  async startRecording(targetUrl: string) {
    await this.navigateToRecording();
    
    await this.page.getByTestId('recording-target-url-input').fill(targetUrl);
    await this.page.getByTestId('start-recording-button').click();
    
    await expect(this.page.getByTestId('recording-status')).toContainText('Recording');
  }

  async stopRecording() {
    await this.page.getByTestId('stop-recording-button').click();
    await expect(this.page.getByTestId('recording-status')).toContainText('Stopped');
  }

  // Error handling helpers
  async waitForToast(message: string, timeoutMs: number = 5000) {
    await expect(this.page.getByText(message)).toBeVisible({ timeout: timeoutMs });
  }

  async waitForErrorToast(timeoutMs: number = 5000) {
    await expect(this.page.getByTestId('error-toast')).toBeVisible({ timeout: timeoutMs });
  }

  async waitForSuccessToast(timeoutMs: number = 5000) {
    await expect(this.page.getByTestId('success-toast')).toBeVisible({ timeout: timeoutMs });
  }

  // Utility methods
  async takeScreenshot(name: string) {
    await this.page.screenshot({ path: `test-results/${name}.png` });
  }

  async getPageTitle(): Promise<string> {
    return await this.page.title();
  }

  async getCurrentUrl(): Promise<string> {
    return this.page.url();
  }
}