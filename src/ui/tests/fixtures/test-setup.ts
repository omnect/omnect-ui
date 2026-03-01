import { Page, expect } from '@playwright/test';
import { mockConfig, mockLoginSuccess, mockRequireSetPassword } from './mock-api';

interface SetupOptions {
  // When false, App.vue will not suppress the factory-reset result modal watcher.
  // Defaults to true so that most tests never accidentally show the modal.
  factoryResetResultAcked?: boolean;
  // When false, App.vue will not suppress the update validation modal watcher.
  // Defaults to true.
  updateValidationAcked?: boolean;
}

export async function setupAndLogin(page: Page, options: SetupOptions = {}) {
  const { factoryResetResultAcked = true, updateValidationAcked = true } = options;

  await mockConfig(page);
  await mockLoginSuccess(page);
  await mockRequireSetPassword(page);

  // Mock healthcheck to avoid errors on app load
  await page.route('**/healthcheck', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
          versionInfo: { current: '1.0.0', required: '1.0.0', mismatch: false },
          updateValidationStatus: { status: 'NoUpdate' },
          networkRollbackOccurred: false,
          factoryResetResultAcked,
          updateValidationAcked
      })
    });
  });

  // Mock initial network config to avoid errors
  await page.route('**/network', async (route) => {
      if (route.request().method() === 'GET') {
          await route.fulfill({
              status: 200,
              body: JSON.stringify({ interfaces: [] })
          });
      } else {
          await route.continue();
      }
  });

  await page.goto('/');
  
  // Perform login
  await page.getByPlaceholder(/enter your password/i).fill('password');
  await page.getByRole('button', { name: /log in/i }).click();
  
  // Wait for dashboard
  await expect(page.getByText('Common Info')).toBeVisible();
}
