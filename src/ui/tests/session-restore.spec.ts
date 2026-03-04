import { test, expect } from '@playwright/test';
import { mockTokenRefresh } from './fixtures/mock-api';
import { setupAndLogin } from './fixtures/test-setup';

test.describe('Session restore on page refresh', () => {
  test('dashboard stays visible after reload when session is valid', async ({ page }) => {
    await setupAndLogin(page);

    // Register the token/refresh mock before reload so initializeCore() can restore the session.
    await mockTokenRefresh(page, 200);

    await page.reload();

    await expect(page.getByText('Common Info')).toBeVisible();
  });

  test('redirects to login after reload when session has expired', async ({ page }) => {
    await setupAndLogin(page);

    // Token refresh returns 401 — simulate an expired or missing server-side session.
    await mockTokenRefresh(page, 401);

    await page.reload();

    await expect(page.getByPlaceholder(/enter your password/i)).toBeVisible();
  });
});
