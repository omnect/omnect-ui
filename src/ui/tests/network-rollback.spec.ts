import { test, expect, Page } from '@playwright/test';
import { publishToCentrifugo } from './fixtures/centrifugo';
import { mockConfig, mockLoginSuccess, mockRequireSetPassword } from './fixtures/mock-api';
import { NetworkTestHarness } from './fixtures/network-test-harness';

// Helper to navigate to network adapter settings
const navigateToAdapter = async (page: Page, adapterName: string) => {
  await page.getByText('Network').click();
  await page.getByText(adapterName).click();
};

test.describe('Network Rollback Status', () => {
  test('rollback status is cleared after ack and does not reappear on re-login', async ({ page }) => {
    let healthcheckRollbackStatus = true;
    const originalIp = '192.168.1.100';

    await mockConfig(page);
    await mockLoginSuccess(page);
    await mockRequireSetPassword(page);

    await page.route('**/healthcheck', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          version_info: { required: '>=0.39.0', current: '0.40.0', mismatch: false },
          update_validation_status: { status: 'valid' },
          network_rollback_occurred: healthcheckRollbackStatus,
        }),
      });
    });

    await page.route('**/ack-rollback', async (route) => {
      if (route.request().method() === 'POST') {
        healthcheckRollbackStatus = false;
        await route.fulfill({ status: 200 });
      }
    });

    await page.goto('/');
    await expect(page.getByText('Network Settings Rolled Back')).toBeVisible({ timeout: 10000 });

    await page.getByRole('button', { name: /ok/i }).click();
    await expect(page.getByText('Network Settings Rolled Back')).not.toBeVisible();
    await page.waitForTimeout(500);

    await page.getByPlaceholder(/enter your password/i).fill('password');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByText('Common Info')).toBeVisible({ timeout: 10000 });

    const adapterConfig = {
      name: 'eth0',
      mac: '00:11:22:33:44:55',
      online: true,
      ipv4: {
        addrs: [{ addr: originalIp, dhcp: false, prefix_len: 24 }],
        dns: ['8.8.8.8'],
        gateways: ['192.168.1.1'],
      },
    };

    await publishToCentrifugo('NetworkStatusV1', { network_status: [adapterConfig] });
    await navigateToAdapter(page, 'eth0'); // Use helper
    await expect(page.getByText('eth0')).toBeVisible();

    await page.reload();
    await expect(page.getByText('Network Settings Rolled Back')).not.toBeVisible({ timeout: 3000 });

    await page.getByPlaceholder(/enter your password/i).fill('password');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByText('Common Info')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Network Settings Rolled Back')).not.toBeVisible();

    await publishToCentrifugo('NetworkStatusV1', { network_status: [adapterConfig] });
    await navigateToAdapter(page, 'eth0'); // Use helper
    await expect(page.getByText('eth0')).toBeVisible();
  });
});

test.describe('Network Rollback Defaults', () => {
  let harness: NetworkTestHarness;

  test.beforeEach(async ({ page }) => {
    harness = new NetworkTestHarness();
    await mockConfig(page);
    await mockLoginSuccess(page);
    await mockRequireSetPassword(page);
    await harness.mockNetworkConfig(page);
    await harness.mockHealthcheck(page);

    await page.goto('/');
    await page.getByPlaceholder(/enter your password/i).fill('password');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByText('Common Info')).toBeVisible();
  });

  test.afterEach(() => {
    harness.reset();
  });

  const setupAdapter = async (page: Page, ip: string, dhcp: boolean) => {
    await harness.publishNetworkStatus([
      harness.createAdapter('eth0', {
        ipv4: {
          addrs: [{ addr: ip, dhcp, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.1.1'],
        },
      }),
    ]);
    await navigateToAdapter(page, 'eth0');
  };

  test('Static -> DHCP: Rollback should be DISABLED by default', async ({ page }) => {
    await setupAdapter(page, 'localhost', false);
    await expect(page.getByLabel('Static')).toBeChecked();

    await page.getByLabel('DHCP').click({ force: true });
    await page.getByRole('button', { name: /save/i }).click();

    await expect(page.getByText('Confirm Network Configuration Change')).toBeVisible();
    await expect(page.getByRole('checkbox', { name: /Enable automatic rollback/i })).not.toBeChecked();
  });

  test('DHCP -> Static: Rollback should be ENABLED by default', async ({ page }) => {
    await setupAdapter(page, 'localhost', true);
    await expect(page.getByLabel('DHCP')).toBeChecked();

    await page.getByLabel('Static').click({ force: true });
    await page.getByRole('textbox', { name: /IP Address/i }).fill('192.168.1.150');
    await page.getByRole('button', { name: /save/i }).click();

    await expect(page.getByText('Confirm Network Configuration Change')).toBeVisible();
    await expect(page.getByRole('checkbox', { name: /Enable automatic rollback/i })).toBeChecked();
  });

  test('DHCP -> Static (Same IP): Rollback should be ENABLED', async ({ page }) => {
    await setupAdapter(page, 'localhost', true);
    await expect(page.getByLabel('DHCP')).toBeChecked();

    await page.getByLabel('Static').click({ force: true });
    // IP is auto-filled with current IP ('localhost'), do NOT change it.
    await page.getByRole('button', { name: /save/i }).click();

    // Verify Modal
    await expect(page.getByText('Confirm Network Configuration Change')).toBeVisible();

    // Verify Checkbox is CHECKED
    await expect(page.getByRole('checkbox', { name: /Enable automatic rollback/i })).toBeChecked();

    // Apply changes
    await page.getByRole('button', { name: /apply changes/i }).click();

    // Verify overlay appears with countdown label
    await expect(page.locator('#overlay').getByText('Automatic rollback in:')).toBeVisible({ timeout: 10000 });
  });

  test('Rollback should show MODAL not SNACKBAR when connection is restored at old IP', async ({ page }) => {
    await setupAdapter(page, 'localhost', false);

    await page.getByLabel('DHCP').click({ force: true });
    
    // Mock /network to return a short rollback timeout for testing
    await harness.mockNetworkConfig(page, { rollbackTimeoutSeconds: 2 });

    await page.getByRole('button', { name: /save/i }).click();
    await expect(page.getByText('Confirm Network Configuration Change')).toBeVisible();
    await page.getByRole('checkbox', { name: /Enable automatic rollback/i }).check();
    
    // Override healthcheck mock to return network_rollback_occurred: true
    await page.unroute('**/healthcheck');
    await page.route('**/healthcheck', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          version_info: { required: '>=0.39.0', current: '0.40.0', mismatch: false },
          update_validation_status: { status: 'valid' },
          network_rollback_occurred: true,
        }),
      });
    });

    await page.getByRole('button', { name: /Apply Changes/i }).click();
    await expect(page.getByText('Applying network settings')).toBeVisible();
    
    await expect(page.getByText('Automatic network rollback successful')).not.toBeVisible();
    await expect(page.getByText('The network settings were rolled back to the previous configuration')).toBeVisible({ timeout: 10000 });
  });
});

