import { Page, expect } from '@playwright/test';
import { mockConfig, mockLoginSuccess, mockRequireSetPassword } from './mock-api';

interface SetupOptions {
  // When false, the factory-reset result and update-validation modals are not suppressed.
  // Defaults to true so that most tests never accidentally show the modals.
  factoryResetResultAcked?: boolean;
  // When false, App.vue will not suppress the update validation modal watcher.
  // Defaults to true.
  updateValidationAcked?: boolean;
}

/**
 * Mock the /healthcheck endpoint with a basic successful response.
 * Suitable for tests that don't exercise version-mismatch or rollback logic.
 *
 * updateValidationAcked defaults to true so that Core's combined watcher suppresses
 * the update-validation modal in tests that don't exercise that flow. Without this,
 * Core defaults the field to false which would override the sessionStorage flag and
 * show the modal when Centrifugo replays a Recovered status from a prior test.
 */
export async function mockHealthcheck(page: Page, options: { updateValidationAcked?: boolean } = {}): Promise<void> {
  const { updateValidationAcked = true } = options;
  await page.route('**/healthcheck', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        versionInfo: { current: '1.0.0', required: '1.0.0', mismatch: false },
        updateValidationStatus: { status: 'NoUpdate' },
        networkRollbackOccurred: false,
        updateValidationAcked,
      }),
    });
  });
}

export async function setupAndLogin(page: Page, options: SetupOptions = {}) {
  const { factoryResetResultAcked = true, updateValidationAcked = true } = options;

  // Set sessionStorage flags before navigation to suppress modals in tests that don't
  // test the factory reset / update validation flow.
  await page.addInitScript((flags) => {
    if (flags.factoryResetResultAcked) sessionStorage.setItem('factoryResetResultAcked', 'true')
    if (flags.updateValidationAcked) sessionStorage.setItem('updateValidationAcked', 'true')
  }, { factoryResetResultAcked, updateValidationAcked })

  await mockConfig(page);
  await mockLoginSuccess(page);
  await mockRequireSetPassword(page);
  await mockHealthcheck(page, { updateValidationAcked });

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
