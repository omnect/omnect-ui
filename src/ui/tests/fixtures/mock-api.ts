import { Page } from '@playwright/test';
import jwt from 'jsonwebtoken';

export async function mockConfig(page: Page) {
  const config = {
    KEYCLOAK_URL: 'http://localhost:8080',
    REALM: 'omnect',
    CLIENT_ID: 'omnect-ui',
  };

  // Add as init script so it's available even before config.js loads
  await page.addInitScript((cfg) => {
    (window as any).__APP_CONFIG__ = cfg;
  }, config);

  // Still mock the file request to avoid 404s
  await page.route('**/config.js', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/javascript',
      body: `window.__APP_CONFIG__ = ${JSON.stringify(config)};`,
    });
  });
}

export async function mockLoginSuccess(page: Page) {
  const token = jwt.sign({ sub: 'user123' }, 'secret', { expiresIn: '1h' });
  await page.route('**/token/login', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'text/plain',
      body: token,
      // Plant the session cookie so that subsequent token/refresh calls in the
      // same browser context include it — matching real backend behavior where
      // actix-session sets Set-Cookie on every successful login.
      headers: {
        'Set-Cookie': 'omnect-ui-session=mock-session-value; Path=/; HttpOnly; SameSite=Strict',
      },
    });
  });
}

export async function mockRequireSetPassword(page: Page) {
  await page.route('**/require-set-password', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: 'false',
    });
  });
}

export async function mockSetPasswordSuccess(page: Page) {
  const token = jwt.sign({ sub: 'user123' }, 'secret', { expiresIn: '1h' });
  await page.route('**/set-password', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({
        status: 200,
        contentType: 'text/plain',
        body: token,
      });
    } else {
      await route.continue();
    }
  });
}

export async function mockUpdatePasswordSuccess(page: Page) {
  await page.route('**/update-password', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({}),
    });
  });
}

export async function mockPortalAuth(page: Page) {
  const user = {
    id_token: 'mock-id-token',
    session_state: 'mock-session-state',
    access_token: 'mock-access-token',
    refresh_token: 'mock-refresh-token',
    token_type: 'Bearer',
    scope: 'openid profile email',
    profile: {
      sub: 'mock-user-id',
      email: 'user@example.com',
      preferred_username: 'user',
      name: 'Mock User',
    },
    expires_at: Math.floor(Date.now() / 1000) + 3600,
  };

  const key = 'oidc.user:http://localhost:8080:omnect-ui';

  await page.addInitScript(({ key, user }) => {
    window.localStorage.setItem(key, JSON.stringify(user));
  }, { key, user });

  // The requiresPortalAuth router guard calls token/validate to establish the
  // backend session flag. Mock it to succeed so tests can reach /set-password.
  await page.route('**/token/validate', async (route) => {
    await route.fulfill({ status: 200 });
  });
}

export async function mockLoginFailure(page: Page, message = 'invalid credentials') {
  await page.route('**/token/login', async (route) => {
    await route.fulfill({
      status: 401,
      contentType: 'text/plain',
      body: message,
    });
  });
}

export async function mockSetPasswordFailure(page: Page, message = 'failed to set password') {
  await page.route('**/set-password', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({
        status: 400,
        contentType: 'text/plain',
        body: message,
      });
    } else {
      await route.continue();
    }
  });
}

export async function mockUpdatePasswordFailure(page: Page, message = 'current password is not correct') {
  await page.route('**/update-password', async (route) => {
    await route.fulfill({
      status: 400,
      contentType: 'text/plain',
      body: message,
    });
  });
}

export async function mockTokenRefresh(page: Page, status = 200) {
  const token = jwt.sign({ sub: 'user123' }, 'secret', { expiresIn: '1h' });
  await page.route('**/token/refresh', async (route) => {
    if (route.request().method() === 'GET') {
      // Mirror the real backend: AuthMw validates the session cookie before
      // issuing a token. Return 401 when the cookie is absent, catching bugs
      // where the login response failed to plant it or the browser failed to
      // send it on the refresh request.
      const cookieHeader = route.request().headers()['cookie'] ?? '';
      const hasSession = cookieHeader.includes('omnect-ui-session=');
      const effectiveStatus = status === 200 && !hasSession ? 401 : status;
      await route.fulfill({
        status: effectiveStatus,
        contentType: 'text/plain',
        body: effectiveStatus === 200 ? token : '',
      });
    } else {
      await route.continue();
    }
  });
}

export interface TimeoutSettingsPayload {
  rebootTimeoutSecs: number;
  factoryResetTimeoutSecs: number;
  firmwareUpdateTimeoutSecs: number;
  networkRollbackTimeoutSecs: number;
}

export const DEFAULT_TIMEOUT_SETTINGS: TimeoutSettingsPayload = {
  rebootTimeoutSecs: 300,
  factoryResetTimeoutSecs: 600,
  firmwareUpdateTimeoutSecs: 900,
  networkRollbackTimeoutSecs: 90,
};

export async function mockGetSettings(page: Page, settings: TimeoutSettingsPayload = DEFAULT_TIMEOUT_SETTINGS) {
  await page.route('**/api/settings', async (route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(settings),
      });
    } else {
      await route.continue();
    }
  });
}

export async function mockSaveSettingsSuccess(page: Page) {
  await page.route('**/api/settings', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({ status: 200, body: '' });
    } else {
      await route.continue();
    }
  });
}

export async function mockSaveSettingsFailure(page: Page, message = 'failed to save settings') {
  await page.route('**/api/settings', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({ status: 500, contentType: 'text/plain', body: message });
    } else {
      await route.continue();
    }
  });
}

export async function mockNetworkConfig(page: Page) {
  // Mock the network configuration endpoint
  await page.route('**/network', async (route) => {
    if (route.request().method() === 'POST') {
        // Mock successful application of network config
        await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
                rollbackTimeoutSeconds: 90,
                uiPort: 5173,
                rollbackEnabled: true
            }),
        });
    } else {
        // Fallback for other methods if any
        await route.continue();
    }
  });
}