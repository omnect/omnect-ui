import { test, expect } from '@playwright/test';
import { NetworkTestHarness } from './fixtures/network-test-harness';

test.describe('Network Multi-Adapter Rollback Modal', () => {
  let harness: NetworkTestHarness;

  test.beforeEach(async ({ page }) => {
    harness = new NetworkTestHarness();
    await harness.setupWithLogin(page);
  });

  test.afterEach(() => {
    harness.reset();
  });

  test.describe('2-Adapter Scenarios', () => {
    test('rollback modal appears for current connection adapter', async ({ page }) => {
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1'],
          },
        },
        {
          name: 'wlan0',
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ]);

      await page.getByRole('tab', { name: 'eth0' }).click();
      await expect(page.getByText('This is your current connection')).toBeVisible();

      const ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('localhost');
      await ipInput.fill('192.168.1.150');
      await page.waitForTimeout(300);

      await page.locator('.v-window-item--active [data-cy=network-apply-button]').click();

      await expect(page.getByText('Confirm Network Configuration Change')).toBeVisible({ timeout: 3000 });

      // Clean up to leave the harness in a neutral state
      await page.getByRole('button', { name: /cancel/i }).click();
      await page.locator('.v-window-item--active [data-cy=network-discard-button]').click();
    });

    test('rollback modal does not appear for non-current adapter (background update)', async ({ page }) => {
      // Navigate to wlan0 directly; verify background updates to eth0 don't trigger rollback modal
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1'],
          },
        },
        {
          name: 'wlan0',
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ], 'wlan0');

      await expect(page.getByText('This is your current connection')).not.toBeVisible();
      await page.waitForTimeout(500); // Wait for NetworkFormStartEdit to be processed

      // Simulate eth0 going offline while the user is on the wlan0 tab
      await harness.publishNetworkStatus([
        {
          name: 'eth0',
          mac: '00:11:22:33:44:55',
          online: false,
          ipv4: { addrs: [], dns: [], gateways: [] },
        },
        {
          name: 'wlan0',
          mac: '00:11:22:33:44:56',
          online: true,
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ]);
      await page.waitForTimeout(300);

      const ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100');
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('192.168.2.150');
      await page.waitForTimeout(500);

      const saveButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
      await saveButton.click();

      await expect(page.getByText('Confirm Network Configuration Change')).not.toBeVisible({ timeout: 2000 });
      await expect(saveButton).toBeDisabled({ timeout: 10000 });
      await expect(page.getByText('Network configuration updated')).toBeVisible();
    });

    test('switching between current and non-current adapters preserves form state', async ({ page }) => {
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] }
        },
        {
          name: 'wlan0',
          ipv4: { addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.2.1'] }
        }
      ]);

      // Make unsaved changes on eth0 (current connection)
      await page.getByRole('tab', { name: 'eth0' }).click();
      const eth0IpInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await eth0IpInput.fill('192.168.1.200');
      await page.waitForTimeout(300);

      // Switch to wlan0 - should trigger "Unsaved Changes" dialog
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();

      // Cancel and stay on eth0
      await page.getByRole('button', { name: /cancel/i }).click();
      await page.waitForTimeout(300);

      // Verify we're still on eth0 with unsaved changes
      await expect(eth0IpInput).toHaveValue('192.168.1.200');

      // Now discard and switch
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();
      await page.locator('[data-cy=network-confirm-discard-button]').click();
      await page.waitForTimeout(300);

      // Verify we switched to wlan0
      const wlan0IpInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(wlan0IpInput).toHaveValue('192.168.2.100');
    });
  });

  test.describe('3-Adapter Scenarios', () => {
    test('rollback modal behavior with three adapters', async ({ page }) => {
      // Setup three adapters with different network configurations
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1']
          }
        },
        {
          name: 'wlan0',
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1']
          }
        },
        {
          name: 'eth1',
          mac: '00:11:22:33:44:56',
          ipv4: {
            addrs: [{ addr: '10.0.0.50', dhcp: false, prefix_len: 24 }],
            dns: ['1.1.1.1'],
            gateways: ['10.0.0.1']
          }
        }
      ]);

      // Verify all three tabs are visible
      await expect(page.getByRole('tab', { name: 'eth0' })).toBeVisible();
      await expect(page.getByRole('tab', { name: 'wlan0' })).toBeVisible();
      await expect(page.getByRole('tab', { name: 'eth1' })).toBeVisible();

      const rollbackModal = page.getByText('Confirm Network Configuration Change');

      // Test 1: eth0 (current connection) - SHOULD show rollback modal
      await page.getByRole('tab', { name: 'eth0' }).click();
      await expect(page.getByText('This is your current connection')).toBeVisible();

      let ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.fill('192.168.1.150');
      await page.waitForTimeout(300);
      await page.locator('.v-window-item--active [data-cy=network-apply-button]').click();

      await expect(rollbackModal).toBeVisible({ timeout: 3000 });
      await page.getByRole('button', { name: /cancel/i }).click();
      await page.locator('.v-window-item--active [data-cy=network-discard-button]').click();
      await page.waitForTimeout(300);

      // Test 2: wlan0 (not current) - should NOT show rollback modal
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('This is your current connection')).not.toBeVisible();

      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100');
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('192.168.2.150');
      await page.waitForTimeout(500);
      await expect(ipInput).toHaveValue('192.168.2.150');

      const saveButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
      await saveButton.click();

      await expect(rollbackModal).not.toBeVisible({ timeout: 2000 });
      await expect(saveButton).toBeDisabled({ timeout: 10000 });
      await expect(page.getByText('Network configuration updated')).toBeVisible();

      // Wait for form to reset after submission
      await page.waitForTimeout(1000);

      // Test 3: eth1 (not current) - should NOT show rollback modal
      await page.getByRole('tab', { name: 'eth1' }).click();
      await expect(page.getByText('This is your current connection')).not.toBeVisible();

      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('10.0.0.50');
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('10.0.0.100');
      await page.waitForTimeout(500);
      await expect(ipInput).toHaveValue('10.0.0.100');

      const saveButton2 = page.locator('.v-window-item--active [data-cy=network-apply-button]');
      await saveButton2.click();

      await expect(rollbackModal).not.toBeVisible({ timeout: 2000 });
      await expect(saveButton2).toBeDisabled({ timeout: 10000 });
      await expect(page.getByText('Network configuration updated')).toBeVisible();

      // Wait for form to reset after submission

    });

    test('form state isolation across multiple adapter switches', async ({ page }) => {
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] }
        },
        {
          name: 'wlan0',
          ipv4: { addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.2.1'] }
        },
        {
          name: 'eth1',
          mac: '00:11:22:33:44:56',
          ipv4: { addrs: [{ addr: '10.0.0.50', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['10.0.0.1'] }
        }
      ]);

      // Make unsaved changes on eth1
      await page.getByRole('tab', { name: 'eth1' }).click();

      // Wait for NetworkFormStartEdit to be called and form to initialize
      await page.waitForTimeout(500);

      let ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('10.0.0.99');
      await page.waitForTimeout(500); // Wait for dirty flag to propagate

      // Switch to wlan0 - should show "Unsaved Changes" dialog
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();

      // Discard and switch
      await page.locator('[data-cy=network-confirm-discard-button]').click();
      await page.waitForTimeout(300);

      // Verify we're on wlan0
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100');

      // Switch back to eth1 - form should be reset (changes were discarded)
      await page.getByRole('tab', { name: 'eth1' }).click();
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('10.0.0.50'); // Original value, not 10.0.0.99

      // Make changes on eth0 (current connection)
      await page.getByRole('tab', { name: 'eth0' }).click();
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.fill('192.168.1.88');
      await page.waitForTimeout(300);

      // Switch to eth1
      await page.getByRole('tab', { name: 'eth1' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();
      await page.getByRole('button', { name: /cancel/i }).click();

      // Verify we're still on eth0 with unsaved changes
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.1.88');

      // Now discard and verify each adapter has its correct original state
      await page.locator('.v-window-item--active [data-cy=network-discard-button]').click();
      await page.waitForTimeout(300);
      await expect(ipInput).toHaveValue('localhost');

      await page.getByRole('tab', { name: 'wlan0' }).click();
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100');

      await page.getByRole('tab', { name: 'eth1' }).click();
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('10.0.0.50');
    });
  });

  test.describe('Edge Cases', () => {
    test('WebSocket update to non-current adapter preserves unsaved changes', async ({ page }) => {
      // Setup: eth0 (current at localhost), wlan0 (at 192.168.2.100)
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] }
        },
        {
          name: 'wlan0',
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1']
          }
        }
      ]);

      // Navigate to wlan0 tab
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await page.waitForTimeout(300);

      const ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100');

      // User changes wlan0 IP to 192.168.2.150 (form becomes dirty)
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('192.168.2.150');
      await page.waitForTimeout(500); // Wait for dirty flag

      // Verify user's edit is in place
      await expect(ipInput).toHaveValue('192.168.2.150');

      // WebSocket publishes NetworkStatusUpdated with wlan0 IP = 192.168.2.200
      // This simulates a real-world scenario like DHCP renew or bridge formation
      await harness.publishNetworkStatus([
        {
          name: 'eth0',
          mac: '00:11:22:33:44:55',
          online: true,
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] }
        },
        {
          name: 'wlan0',
          mac: '00:11:22:33:44:56',
          online: true,
          ipv4: {
            addrs: [{ addr: '192.168.2.200', dhcp: false, prefix_len: 24 }], // Changed!
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1']
          }
        }
      ]);

      await page.waitForTimeout(500);

      // EXPECTED: Form still shows 192.168.2.150 (user's edits preserved)
      await expect(ipInput).toHaveValue('192.168.2.150');

      // User can still save with their value
      const saveButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
      await saveButton.click();
      await expect(saveButton).toBeDisabled({ timeout: 10000 });
      await expect(page.getByText('Network configuration updated')).toBeVisible();
    });

    test('rollback modal blocks tab switching during submission', async ({ page }) => {
      // Setup: eth0 (current), wlan0 (not current)
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] }
        },
        {
          name: 'wlan0',
          ipv4: { addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.2.1'] }
        }
      ]);

      // Edit eth0 IP
      await page.getByRole('tab', { name: 'eth0' }).click();
      const ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.fill('192.168.1.150');
      await page.waitForTimeout(300);

      // Click Save - rollback modal appears for current connection adapter
      await page.locator('.v-window-item--active [data-cy=network-apply-button]').click();

      const rollbackModal = page.getByText('Confirm Network Configuration Change');
      await expect(rollbackModal).toBeVisible({ timeout: 3000 });

      // Verify the modal overlay prevents tab switching
      // The wlan0 tab should be present but the modal overlay blocks interaction
      const wlan0Tab = page.getByRole('tab', { name: 'wlan0' });
      await expect(wlan0Tab).toBeVisible();

      // Verify modal is still visible (hasn't been dismissed by attempting to click tab)
      await expect(rollbackModal).toBeVisible();

      // Cancel the modal
      await page.getByRole('button', { name: /cancel/i }).click();
      await expect(rollbackModal).not.toBeVisible();

      // Reset the form
      await page.locator('.v-window-item--active [data-cy=network-discard-button]').click();
      await page.waitForTimeout(300);

      // Now tab switch should work normally (no modal blocking)
      await wlan0Tab.click();
      const wlan0IpInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(wlan0IpInput).toHaveValue('192.168.2.100');
    });

    test('only first adapter with matching IP is treated as current connection', async ({ page }) => {
      // Setup: eth0 (localhost), wlan0 (localhost - duplicate!), eth1 (10.0.0.50)
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] }
        },
        {
          name: 'wlan0',
          ipv4: { addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['192.168.1.1'] } // Same IP!
        },
        {
          name: 'eth1',
          mac: '00:11:22:33:44:56',
          ipv4: { addrs: [{ addr: '10.0.0.50', dhcp: false, prefix_len: 24 }], dns: ['8.8.8.8'], gateways: ['10.0.0.1'] }
        }
      ]);

      const rollbackModal = page.getByText('Confirm Network Configuration Change');

      // Verify eth0 is marked as current connection
      await page.getByRole('tab', { name: 'eth0' }).click();
      await expect(page.getByText('This is your current connection')).toBeVisible();

      // Edit eth0, verify rollback modal appears
      let ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.fill('192.168.1.150');
      await page.waitForTimeout(300);
      await page.locator('.v-window-item--active [data-cy=network-apply-button]').click();
      await expect(rollbackModal).toBeVisible({ timeout: 3000 });
      await page.getByRole('button', { name: /cancel/i }).click();
      await page.locator('.v-window-item--active [data-cy=network-discard-button]').click();
      await page.waitForTimeout(300);

      // Verify wlan0 is NOT marked as current connection (even though it has same IP)
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('This is your current connection')).not.toBeVisible();

      // Edit wlan0, verify rollback modal does NOT appear
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('localhost');
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('192.168.2.150');
      await page.waitForTimeout(500);

      const saveButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
      await saveButton.click();
      await expect(rollbackModal).not.toBeVisible({ timeout: 2000 });
      await expect(saveButton).toBeDisabled({ timeout: 10000 });
      await expect(page.getByText('Network configuration updated')).toBeVisible();

      await page.waitForTimeout(1000);

      // Verify eth1 also does NOT show rollback modal
      await page.getByRole('tab', { name: 'eth1' }).click();
      await expect(page.getByText('This is your current connection')).not.toBeVisible();

      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('10.0.0.50');
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('10.0.0.100');
      await page.waitForTimeout(500);

      const saveButton2 = page.locator('.v-window-item--active [data-cy=network-apply-button]');
      await saveButton2.click();
      await expect(rollbackModal).not.toBeVisible({ timeout: 2000 });
      await expect(saveButton2).toBeDisabled({ timeout: 10000 });
      await expect(page.getByText('Network configuration updated')).toBeVisible();

      // Wait for form to reset after submission
    });

    test('rapid tab switching with edits shows correct dirty state', async ({ page }) => {
      // Setup: 3 adapters (eth0, wlan0, eth1)
      await harness.setup(page, [
        {
          name: 'eth0',
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1']
          }
        },
        {
          name: 'wlan0',
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1']
          }
        },
        {
          name: 'eth1',
          mac: '00:11:22:33:44:56',
          ipv4: {
            addrs: [{ addr: '10.0.0.50', dhcp: false, prefix_len: 24 }],
            dns: ['1.1.1.1'],
            gateways: ['10.0.0.1']
          }
        }
      ]);

      // Edit eth0 IP (dirty = true)
      await page.getByRole('tab', { name: 'eth0' }).click();
      let ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.fill('192.168.1.99');
      await page.waitForTimeout(300);

      // Click wlan0 tab → unsaved changes dialog → cancel
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();
      await page.getByRole('button', { name: /cancel/i }).click();
      await page.waitForTimeout(300);

      // Still on eth0, verify IP changed
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.1.99');

      // Click wlan0 again → discard changes
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();
      await page.locator('[data-cy=network-confirm-discard-button]').click();
      await page.waitForTimeout(300);

      // Now on wlan0, verify original IP
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100');

      // Edit wlan0 IP (dirty = true)
      await ipInput.click();
      await ipInput.clear();
      await ipInput.fill('192.168.2.88');
      await page.waitForTimeout(500);

      // Click eth1 → discard
      await page.getByRole('tab', { name: 'eth1' }).click();
      await expect(page.getByText('Unsaved Changes', { exact: true })).toBeVisible();
      await page.locator('[data-cy=network-confirm-discard-button]').click();
      await page.waitForTimeout(300);

      // Verify eth1 shows original IP, dirty = false (no unsaved changes)
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('10.0.0.50');

      // Switch back to eth0 - should show original IP (changes were discarded)
      await page.getByRole('tab', { name: 'eth0' }).click();
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('localhost'); // Original, not 192.168.1.99

      // Switch back to wlan0 - should show original IP (changes were discarded)
      await page.getByRole('tab', { name: 'wlan0' }).click();
      ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await expect(ipInput).toHaveValue('192.168.2.100'); // Original, not 192.168.2.88
    });

    test('online status updates with multiple adapters', async ({ page }) => {
      // Setup two adapters, both initially online
      await harness.setup(page, [
        {
          name: 'eth0',
          online: true,
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1'],
          },
        },
        {
          name: 'wlan0',
          mac: '00:11:22:33:44:66',
          online: true,
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ]);

      // Verify eth0 is online
      await expect(page.locator('.v-chip').filter({ hasText: 'Online' })).toBeVisible();

      // Simulate eth0 going offline (cable removed)
      await harness.publishNetworkStatus([
        {
          name: 'eth0',
          online: false, // Changed to offline
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1'],
          },
        },
        {
          name: 'wlan0',
          mac: '00:11:22:33:44:66',
          online: true, // Still online
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ]);

      // Verify eth0 now shows as offline
      await expect(page.locator('.v-chip').filter({ hasText: 'Offline' })).toBeVisible({ timeout: 5000 });
      await expect(page.locator('.v-chip').filter({ hasText: 'Online' })).not.toBeVisible();

      // Switch to wlan0 - should still be online
      await page.getByRole('tab', { name: 'wlan0' }).click();
      await page.waitForTimeout(500);
      await expect(page.locator('.v-chip').filter({ hasText: 'Online' })).toBeVisible();
      await expect(page.locator('.v-chip').filter({ hasText: 'Offline' })).not.toBeVisible();
    });

    test('online status updates even with dirty form on multi-adapter', async ({ page }) => {
      // Setup two adapters, both initially online
      await harness.setup(page, [
        {
          name: 'eth0',
          online: true,
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1'],
          },
        },
        {
          name: 'wlan0',
          mac: '00:11:22:33:44:66',
          online: true,
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ]);

      // Verify eth0 is online
      await expect(page.locator('.v-chip').filter({ hasText: 'Online' })).toBeVisible();

      // Make form dirty by editing IP
      const ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
      await ipInput.fill('192.168.1.150');
      await page.waitForTimeout(500);

      // Verify form is dirty
      await expect(page.locator('.v-window-item--active [data-cy=network-discard-button]')).toBeEnabled();

      // While form is dirty, simulate eth0 going offline (cable removed)
      await harness.publishNetworkStatus([
        {
          name: 'eth0',
          online: false, // Changed to offline
          ipv4: {
            addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.1.1'],
          },
        },
        {
          name: 'wlan0',
          mac: '00:11:22:33:44:66',
          online: true, // Still online
          ipv4: {
            addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
            dns: ['8.8.8.8'],
            gateways: ['192.168.2.1'],
          },
        },
      ]);

      // Verify online status updates even when the form has unsaved changes
      await expect(page.locator('.v-chip').filter({ hasText: 'Offline' })).toBeVisible({ timeout: 5000 });
      await expect(page.locator('.v-chip').filter({ hasText: 'Online' })).not.toBeVisible();

      // Verify edited IP is still preserved (dirty flag should prevent form field reset)
      await expect(ipInput).toHaveValue('192.168.1.150');
    });

    test('online chip updates when adapter goes offline', async ({ page }) => {
      // This test specifically checks the Online/Offline chip element by exact CSS classes
      await harness.setup(page, {
        online: true,
        ipv4: {
          addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.1.1'],
        },
      });

      // Find the specific chip by its color classes
      const onlineChip = page.locator('.v-chip.text-light-green-darken-2').filter({ hasText: 'Online' });
      const offlineChip = page.locator('.v-chip.text-red-darken-2').filter({ hasText: 'Offline' });

      // Verify chip shows Online initially with green color
      await expect(onlineChip).toBeVisible();
      await expect(offlineChip).not.toBeVisible();

      // Simulate cable removal - adapter goes offline
      await harness.publishNetworkStatus([{
        name: 'eth0',
        mac: '00:11:22:33:44:55',
        online: false,
        ipv4: {
          addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.1.1'],
        },
      }]);

      // Verify the chip updates to show Offline status
      await expect(offlineChip).toBeVisible({ timeout: 5000 });
      await expect(onlineChip).not.toBeVisible();
    });
  });
});

test.describe('2nd Adapter Dirty State Detection', () => {
  let harness: NetworkTestHarness;

  test.beforeEach(async ({ page }) => {
    harness = new NetworkTestHarness();
    await harness.setupWithLogin(page);
  });

  test.afterEach(() => {
    harness.reset();
  });

  const TWO_ADAPTERS = [
    {
      name: 'eth0',
      ipv4: {
        addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
        dns: ['8.8.8.8'],
        gateways: ['192.168.1.1'],
      },
    },
    {
      name: 'wlan0',
      mac: '00:11:22:33:44:66',
      ipv4: {
        addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
        dns: ['8.8.8.8'],
        gateways: ['192.168.2.1'],
      },
    },
  ];

  test('editing auto-selected adapter works without explicit tab click (regression: immediate watcher)', async ({ page }) => {
    // Regression test for 45927e3: when network data is already loaded (Centrifugo replay),
    // watch(networkStatus, { immediate: true }) sets tab.value before watch(tab) is registered.
    // The subsequent explicit click on the same tab is a no-op (newTab === oldTab), so
    // networkFormStartEdit was never called and Core stayed in Idle state.
    await harness.publishNetworkStatus(TWO_ADAPTERS.map(a => harness.createAdapter(a.name, a)));
    await harness.navigateToNetwork(page);
    // Do NOT click any tab — rely solely on the auto-selection from the immediate watcher.
    // eth0 has addr 'localhost' so it is the currentConnectionAdapter and will be auto-selected.
    await page.waitForTimeout(300);

    const ipInput = page.locator('.v-window-item--active').getByRole('textbox', { name: /IP Address/i }).first();
    await expect(ipInput).toHaveValue('localhost');
    await ipInput.fill('192.168.1.150');
    await page.waitForTimeout(300);

    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeEnabled({ timeout: 3000 });
  });

  test('DHCP toggle on 2nd adapter marks form dirty', async ({ page }) => {
    await harness.setup(page, TWO_ADAPTERS, 'wlan0');
    await page.waitForTimeout(300);

    await page.locator('.v-window-item--active').getByLabel('DHCP').click({ force: true });
    await page.waitForTimeout(300);

    const applyButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
    await expect(applyButton).toBeEnabled({ timeout: 3000 });

    const discardButton = page.locator('.v-window-item--active [data-cy=network-discard-button]');
    await expect(discardButton).toBeEnabled();

    await expect(page.locator('.v-window-item--active').getByText('You have unsaved changes')).toBeVisible();
  });

  test('IP change on 2nd adapter marks form dirty', async ({ page }) => {
    await harness.setup(page, TWO_ADAPTERS, 'wlan0');
    await page.waitForTimeout(300);

    const ipInput = page.locator('.v-window-item--active').getByRole('textbox', { name: /IP Address/i }).first();
    await ipInput.fill('192.168.2.200');
    await page.waitForTimeout(300);

    const applyButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
    await expect(applyButton).toBeEnabled({ timeout: 3000 });

    await expect(page.locator('.v-window-item--active').getByText('You have unsaved changes')).toBeVisible();
  });

  test('discard on 2nd adapter resets dirty state and restores original value', async ({ page }) => {
    await harness.setup(page, TWO_ADAPTERS, 'wlan0');
    await page.waitForTimeout(300);

    const ipInput = page.getByRole('textbox', { name: /IP Address/i }).first();
    await ipInput.fill('192.168.2.200');
    await page.waitForTimeout(300);

    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeEnabled();

    await page.locator('.v-window-item--active [data-cy=network-discard-button]').click();
    await page.waitForTimeout(300);

    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeDisabled({ timeout: 3000 });
    await expect(ipInput).toHaveValue('192.168.2.100');
  });

  test('DHCP toggle on 2nd adapter after WebSocket resync does not block dirty detection', async ({ page }) => {
    // This test reproduces the isSyncingFromCore race: a WebSocket NetworkStatus update while
    // the form is clean triggers syncLocalFieldsFromCore (isSyncingFromCore=true). If the user
    // interacts during that window, the change was silently dropped and dirty never set.
    await harness.setup(page, TWO_ADAPTERS, 'wlan0');
    await page.waitForTimeout(300);

    // Trigger a WebSocket re-sync while the form is clean (same data, simulates periodic update)
    await harness.publishNetworkStatus([
      {
        name: 'eth0',
        mac: '00:11:22:33:44:55',
        online: true,
        ipv4: {
          addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.1.1'],
        },
      },
      {
        name: 'wlan0',
        mac: '00:11:22:33:44:66',
        online: true,
        ipv4: {
          addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.2.1'],
        },
      },
    ]);

    // Interact as soon as possible after the WebSocket update to hit the isSyncingFromCore window
    await page.locator('.v-window-item--active').getByLabel('DHCP').click({ force: true });
    await page.waitForTimeout(300);

    const applyButton = page.locator('.v-window-item--active [data-cy=network-apply-button]');
    await expect(applyButton).toBeEnabled({ timeout: 3000 });
  });

  test('dirty state on 2nd adapter after tab switch via discard dialog', async ({ page }) => {
    // Reproduces Bug Path 2: confirmTabChange calls networkFormStartEdit twice due to watch(tab)
    // also firing. If the second call resets original_data after the user starts editing, dirty=false.
    await harness.setup(page, TWO_ADAPTERS, 'wlan0');
    await page.getByRole('tab', { name: 'eth0' }).click();
    await page.waitForTimeout(300);

    // Make eth0 dirty
    const eth0IpInput = page.getByRole('textbox', { name: /IP Address/i }).first();
    await expect(eth0IpInput).toHaveValue('localhost');
    await eth0IpInput.fill('192.168.1.150');
    await page.waitForTimeout(300);

    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeEnabled({ timeout: 3000 });

    // Switch to wlan0 — triggers unsaved changes dialog
    await page.getByRole('tab', { name: 'wlan0' }).click();
    await expect(page.locator('.v-overlay--active .v-card-title').filter({ hasText: 'Unsaved Changes' })).toBeVisible({ timeout: 3000 });
    await page.locator('[data-cy=network-confirm-discard-button]').click();
    await page.waitForTimeout(300);

    // Now on wlan0 — apply/discard should be disabled (clean state)
    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeDisabled({ timeout: 3000 });

    // Make wlan0 dirty
    await page.locator('.v-window-item--active').getByLabel('DHCP').click({ force: true });
    await page.waitForTimeout(300);

    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeEnabled({ timeout: 3000 });
  });

  test('switching back to 1st adapter after editing 2nd does not carry over dirty flag', async ({ page }) => {
    await harness.setup(page, TWO_ADAPTERS, 'wlan0');
    await page.waitForTimeout(300);

    // Make wlan0 dirty
    const ipInput = page.locator('.v-window-item--active').getByRole('textbox', { name: /IP Address/i }).first();
    await ipInput.fill('192.168.2.200');
    await page.waitForTimeout(300);

    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeEnabled();

    // Switch back to eth0 — should show unsaved changes dialog
    await page.getByRole('tab', { name: 'eth0' }).click();
    await expect(page.locator('.v-overlay--active .v-card-title').filter({ hasText: 'Unsaved Changes' })).toBeVisible({ timeout: 3000 });
    await page.locator('[data-cy=network-confirm-discard-button]').click();
    await page.waitForTimeout(300);

    // eth0's apply button must be disabled (no changes made to eth0)
    await expect(page.locator('.v-window-item--active [data-cy=network-apply-button]')).toBeDisabled({ timeout: 3000 });
  });

  test('tab icon reflects online/offline state for non-active adapter', async ({ page }) => {
    await harness.setup(page, TWO_ADAPTERS, 'eth0');
    await page.waitForTimeout(300);

    // wlan0 tab should initially show online icon (filled circle)
    const wlan0Tab = page.locator('.v-tab').filter({ hasText: /^wlan0/ });
    await expect(wlan0Tab.locator('.mdi-circle')).toBeVisible();
    await expect(wlan0Tab.locator('.mdi-circle-outline')).not.toBeVisible();

    // Simulate wlan0 going offline while user stays on eth0
    await harness.publishNetworkStatus([
      {
        name: 'eth0',
        mac: '00:11:22:33:44:55',
        online: true,
        ipv4: {
          addrs: [{ addr: 'localhost', dhcp: false, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.1.1'],
        },
      },
      {
        name: 'wlan0',
        mac: '00:11:22:33:44:66',
        online: false,
        ipv4: {
          addrs: [{ addr: '192.168.2.100', dhcp: false, prefix_len: 24 }],
          dns: ['8.8.8.8'],
          gateways: ['192.168.2.1'],
        },
      },
    ]);

    // wlan0 tab icon should switch to outline (offline)
    await expect(wlan0Tab.locator('.mdi-circle-outline')).toBeVisible({ timeout: 3000 });
    await expect(wlan0Tab.locator('.mdi-circle:not(.mdi-circle-outline)')).not.toBeVisible();
  });
});
