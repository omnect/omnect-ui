import { test, expect } from '@playwright/test';
import {
  mockConfig,
  mockLoginSuccess,
  mockLoginFailure,
  mockRequireSetPassword,
  mockSetPasswordSuccess,
  mockSetPasswordFailure,
  mockUpdatePasswordSuccess,
  mockUpdatePasswordFailure,
  mockPortalAuth
} from './fixtures/mock-api';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    await mockConfig(page);
    await mockLoginSuccess(page);

    // Mock logout endpoint
    await page.route('**/logout', async (route) => {
        await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({}),
        });
    });
  });

  test('can login successfully', async ({ page }) => {
    await mockRequireSetPassword(page);
    await page.goto('/');

    // Login
    await page.getByPlaceholder(/enter your password/i).fill('password');
    await page.getByRole('button', { name: /log in/i }).click();

    // Wait for dashboard
    await expect(page.getByText('Common Info')).toBeVisible();
  });

  test('can logout successfully', async ({ page }) => {
    await mockRequireSetPassword(page);
    await page.goto('/');

    // Login
    await page.getByPlaceholder(/enter your password/i).fill('password');
    await page.getByRole('button', { name: /log in/i }).click();

    // Wait for dashboard
    await expect(page.getByText('Common Info')).toBeVisible();

    // Open user menu
    await page.locator('[data-cy="user-menu"]').click();

    // Click logout button
    await page.getByRole('button', { name: /logout/i }).click();

    // Assert redirect to login page
    await expect(page.getByPlaceholder(/enter your password/i)).toBeVisible();
  });

  test('redirects to set-password if required', async ({ page }) => {
    await mockPortalAuth(page);
    // Mock require-set-password returning true
    await page.route('**/require-set-password', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: 'true',
      });
    });

    await page.goto('/');

    // Should be on set-password page
    await expect(page).toHaveURL(/\/set-password/);
    await expect(page.getByText(/set password/i).first()).toBeVisible();
  });

  test('can set initial password successfully', async ({ page }) => {
    await mockPortalAuth(page);
    // Mock require-set-password returning true
    await page.route('**/require-set-password', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: 'true',
      });
    });
    await mockSetPasswordSuccess(page);

    await page.goto('/');

    // Wait for redirect chain (/ → /login → /set-password) to complete
    await expect(page.getByRole('heading', { name: /set password/i })).toBeVisible();

    // Fill set-password form
    await page.locator('input[type="password"]').nth(0).fill('new-password');
    await page.locator('input[type="password"]').nth(1).fill('new-password');
    await page.getByRole('button', { name: /set password/i }).click();

    // SetPasswordResponse now authenticates directly (token in response body),
    // so useAuthNavigation redirects to dashboard immediately.
    await expect(page.getByText('Common Info')).toBeVisible({ timeout: 10000 });
  });

  test('auto-authenticates after setting password via direct navigation', async ({ page }) => {
    // Simulates the real Keycloak flow: after OIDC redirect, the user lands
    // directly on /set-password without going through Login.vue.
    // This means CheckRequiresPasswordSet is never called, so
    // requires_password_set is never set to true in the Core model.
    await mockPortalAuth(page);
    await mockSetPasswordSuccess(page);

    // Navigate directly to /set-password (bypasses Login.vue entirely)
    await page.goto('/set-password');

    await expect(page.getByRole('heading', { name: /set password/i })).toBeVisible();

    // Fill and submit
    await page.locator('input[type="password"]').nth(0).fill('new-password');
    await page.locator('input[type="password"]').nth(1).fill('new-password');
    await page.getByRole('button', { name: /set password/i }).click();

    // Should auto-authenticate via token in response and redirect to dashboard
    await expect(page.getByText('Common Info')).toBeVisible({ timeout: 10000 });
  });

  test('re-triggers OIDC login when backend session is stale', async ({ page }) => {
    // Simulate: OIDC user exists in localStorage but token/validate fails because
    // the backend session was wiped (e.g. after factory reset). The router guard
    // now detects this before showing the form and redirects to Keycloak.
    await mockPortalAuth(page);
    // Override the 200 mock from mockPortalAuth to simulate a stale session.
    await page.route('**/token/validate', async (route) => {
      await route.fulfill({ status: 401 });
    });

    // Intercept the redirect to Keycloak (mocked authority) to verify it's attempted.
    // We expect a navigation away from the app to the Keycloak URL.
    const redirectPromise = page.waitForRequest(req => req.url().includes('localhost:8080'));

    await page.goto('/set-password');

    // Wait for the redirect to be triggered
    await redirectPromise;
  });

  test('re-triggers OIDC login when session expires during submission', async ({ page }) => {
    // Simulate: OIDC user exists in localStorage (router guard passes)
    // but backend rejects because portal_validated session flag is missing.
    // The frontend should clear the stale OIDC user and redirect to Keycloak.
    await mockPortalAuth(page);
    await page.route('**/set-password', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 401,
          contentType: 'text/plain',
          body: 'portal authentication required',
        });
      } else {
        await route.continue();
      }
    });

    // Mock OIDC discovery so signinRedirect() can proceed
    await page.route('**/.well-known/openid-configuration', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          issuer: 'http://localhost:8080',
          authorization_endpoint: 'http://localhost:8080/auth',
          token_endpoint: 'http://localhost:8080/token',
          jwks_uri: 'http://localhost:8080/certs',
          response_types_supported: ['code'],
          subject_types_supported: ['public'],
          id_token_signing_alg_values_supported: ['RS256'],
        }),
      });
    });

    await page.goto('/set-password');
    await expect(page.getByRole('heading', { name: /set password/i })).toBeVisible();

    // Intercept the Keycloak redirect to verify it happens
    const oidcRedirect = page.waitForURL(/localhost:8080/, { timeout: 5000 }).catch(() => null);

    await page.locator('input[type="password"]').nth(0).fill('new-password');
    await page.locator('input[type="password"]').nth(1).fill('new-password');
    await page.getByRole('button', { name: /set password/i }).click();

    // Should redirect to Keycloak for re-authentication
    await oidcRedirect;
    // Page navigated away from the app — either to Keycloak or chrome-error (no real Keycloak)
    await expect(page).not.toHaveURL(/localhost:5173/, { timeout: 5000 });
  });

  test('no auth errors on set-password page when WiFi is available', async ({ page }) => {
    await mockPortalAuth(page);
    await page.route('**/require-set-password', async (route) => {
      await route.fulfill({ status: 200, contentType: 'application/json', body: 'true' });
    });

    // WiFi hardware is available — CheckAvailability (unauthenticated) succeeds
    await page.route('**/wifi/available', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ available: true, interfaceName: 'wlan0' }),
      });
    });

    // Authenticated WiFi endpoints should NOT be called before login.
    // If they are, they return 401 which would surface as an error.
    await page.route('**/wifi/status', async (route) => {
      if (route.request().method() === 'GET') {
        await route.fulfill({ status: 401, contentType: 'text/plain', body: 'Not authenticated' });
      } else {
        await route.continue();
      }
    });
    await page.route('**/wifi/networks', async (route) => {
      if (route.request().method() === 'GET') {
        await route.fulfill({ status: 401, contentType: 'text/plain', body: 'Not authenticated' });
      } else {
        await route.continue();
      }
    });

    await page.goto('/');
    await expect(page.getByRole('heading', { name: /set password/i })).toBeVisible();

    // No auth error should be displayed
    await expect(page.getByText(/not authenticated/i)).not.toBeVisible();
  });

  test('can update password successfully', async ({ page }) => {
    await mockRequireSetPassword(page);
    await mockUpdatePasswordSuccess(page);
    await page.goto('/');

    // Login first
    await page.getByPlaceholder(/enter your password/i).fill('password');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByText('Common Info')).toBeVisible();

    // Navigate to update-password via user menu
    await page.locator('[data-cy="user-menu"]').click();
    await page.getByRole('button', { name: /change password/i }).click();

    // Fill update-password form
    await expect(page.getByText(/update password/i).first()).toBeVisible();

    // Using nth to avoid strict mode violations with Vuetify icons having similar labels/aria-labels
    await page.locator('input[type="password"]').nth(0).fill('password');
    await page.locator('input[type="password"]').nth(1).fill('new-password');
    await page.locator('input[type="password"]').nth(2).fill('new-password');

    await page.getByRole('button', { name: /set new password/i }).click();

    // Verify success message and redirect to dashboard
    await expect(page.getByText(/password updated successfully/i)).toBeVisible();
    await expect(page.getByText('Common Info')).toBeVisible({ timeout: 10000 });
  });

  test.describe('inline error display', () => {
    test('shows inline error on login failure without toast', async ({ page }) => {
      await mockRequireSetPassword(page);
      await mockLoginFailure(page);

      await page.goto('/');
      await page.getByPlaceholder(/enter your password/i).fill('wrong-password');
      await page.getByRole('button', { name: /log in/i }).click();

      await expect(page.getByText('invalid credentials')).toBeVisible();
      await expect(page.locator('.v-snackbar--active')).not.toBeVisible();
    });

    test('shows inline error for password mismatch on set-password', async ({ page }) => {
      await mockPortalAuth(page);
      await page.route('**/require-set-password', async (route) => {
        await route.fulfill({ status: 200, contentType: 'application/json', body: 'true' });
      });

      await page.goto('/');
      await expect(page.getByRole('heading', { name: /set password/i })).toBeVisible();

      await page.locator('input[type="password"]').nth(0).fill('password1');
      await page.locator('input[type="password"]').nth(1).fill('password2');
      await page.getByRole('button', { name: /set password/i }).click();

      await expect(page.getByText('Passwords do not match.')).toBeVisible();
      await expect(page.locator('.v-snackbar--active')).not.toBeVisible();
    });

    test('shows inline error on set-password API failure without toast', async ({ page }) => {
      await mockPortalAuth(page);
      await mockSetPasswordFailure(page);
      await page.route('**/require-set-password', async (route) => {
        await route.fulfill({ status: 200, contentType: 'application/json', body: 'true' });
      });

      await page.goto('/');
      await expect(page.getByRole('heading', { name: /set password/i })).toBeVisible();

      await page.locator('input[type="password"]').nth(0).fill('new-password');
      await page.locator('input[type="password"]').nth(1).fill('new-password');
      await page.getByRole('button', { name: /set password/i }).click();

      await expect(page.getByText('failed to set password')).toBeVisible();
      await expect(page.locator('.v-snackbar--active')).not.toBeVisible();
    });

    test('shows inline error for password mismatch on update-password', async ({ page }) => {
      await mockRequireSetPassword(page);
      await page.goto('/');

      await page.getByPlaceholder(/enter your password/i).fill('password');
      await page.getByRole('button', { name: /log in/i }).click();
      await expect(page.getByText('Common Info')).toBeVisible();

      await page.locator('[data-cy="user-menu"]').click();
      await page.getByRole('button', { name: /change password/i }).click();
      await expect(page.getByText(/update password/i).first()).toBeVisible();

      await page.locator('input[type="password"]').nth(0).fill('current');
      await page.locator('input[type="password"]').nth(1).fill('new-pass1');
      await page.locator('input[type="password"]').nth(2).fill('new-pass2');
      await page.getByRole('button', { name: /set new password/i }).click();

      await expect(page.getByText('Passwords do not match.')).toBeVisible();
      await expect(page.locator('.v-snackbar--active')).not.toBeVisible();
    });

    test('shows inline error on update-password with wrong current password without toast', async ({ page }) => {
      await mockRequireSetPassword(page);
      await mockUpdatePasswordFailure(page);
      await page.goto('/');

      await page.getByPlaceholder(/enter your password/i).fill('password');
      await page.getByRole('button', { name: /log in/i }).click();
      await expect(page.getByText('Common Info')).toBeVisible();

      await page.locator('[data-cy="user-menu"]').click();
      await page.getByRole('button', { name: /change password/i }).click();
      await expect(page.getByText(/update password/i).first()).toBeVisible();

      await page.locator('input[type="password"]').nth(0).fill('wrong-current');
      await page.locator('input[type="password"]').nth(1).fill('new-password');
      await page.locator('input[type="password"]').nth(2).fill('new-password');
      await page.getByRole('button', { name: /set new password/i }).click();

      await expect(page.getByText('current password is not correct')).toBeVisible();
      await expect(page.locator('.v-snackbar--active')).not.toBeVisible();
    });
  });
});
