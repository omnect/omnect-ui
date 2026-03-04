import { test, expect } from '@playwright/test';
import {
	DEFAULT_TIMEOUT_SETTINGS,
	type TimeoutSettingsPayload,
} from './fixtures/mock-api';
import { setupAndLogin } from './fixtures/test-setup';

// Field order matches Settings.vue render order:
// 0: networkRollback, 1: reboot, 2: factoryReset, 3: firmwareUpdate
const inputs = (page: any) => page.locator('input[type="number"]');

async function setupWithSettings(page: any, settings: TimeoutSettingsPayload = DEFAULT_TIMEOUT_SETTINGS) {
	await page.route('**/settings', async (route: any) => {
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

async function navigateToSettings(page: any) {
	// Use in-app navigation to preserve WASM auth state
	await page.locator('[data-cy="main-nav"]').getByText('Settings').click();
	await expect(page.getByText('Operation Timeouts')).toBeVisible({ timeout: 5000 });
}

test.describe('Settings page', () => {
	test('displays timeout values loaded from the backend', async ({ page }) => {
		const custom: TimeoutSettingsPayload = {
			rebootTimeoutSecs: 120,
			factoryResetTimeoutSecs: 300,
			firmwareUpdateTimeoutSecs: 450,
			networkRollbackTimeoutSecs: 60,
		};
		await setupWithSettings(page, custom);
		await setupAndLogin(page);
		await navigateToSettings(page);

		await expect(inputs(page).nth(0)).toHaveValue('60');
		await expect(inputs(page).nth(1)).toHaveValue('120');
		await expect(inputs(page).nth(2)).toHaveValue('300');
		await expect(inputs(page).nth(3)).toHaveValue('450');
	});

	test('save sends POST with modified values and shows success message', async ({ page }) => {
		let capturedBody: TimeoutSettingsPayload | null = null;

		await page.route('**/settings', async (route: any) => {
			if (route.request().method() === 'GET') {
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(DEFAULT_TIMEOUT_SETTINGS),
				});
			} else if (route.request().method() === 'POST') {
				capturedBody = JSON.parse(route.request().postData() ?? '{}');
				await route.fulfill({ status: 200, body: '' });
			} else {
				await route.continue();
			}
		});

		await setupAndLogin(page);
		await navigateToSettings(page);

		await inputs(page).nth(1).fill('180');
		await page.getByRole('button', { name: 'Save' }).click();

		await expect(page.getByText('Settings saved')).toBeVisible({ timeout: 5000 });
		expect(capturedBody).not.toBeNull();
		expect((capturedBody as TimeoutSettingsPayload).rebootTimeoutSecs).toBe(180);

		// Verify the form does not revert to old values after Core re-renders
		await expect(inputs(page).nth(1)).toHaveValue('180');
	});

	test('reset to defaults restores original values', async ({ page }) => {
		const custom: TimeoutSettingsPayload = {
			rebootTimeoutSecs: 120,
			factoryResetTimeoutSecs: 300,
			firmwareUpdateTimeoutSecs: 450,
			networkRollbackTimeoutSecs: 60,
		};
		await setupWithSettings(page, custom);
		await setupAndLogin(page);
		await navigateToSettings(page);

		await page.getByRole('button', { name: 'Reset to defaults' }).click();

		await expect(inputs(page).nth(0)).toHaveValue(String(DEFAULT_TIMEOUT_SETTINGS.networkRollbackTimeoutSecs));
		await expect(inputs(page).nth(1)).toHaveValue(String(DEFAULT_TIMEOUT_SETTINGS.rebootTimeoutSecs));
		await expect(inputs(page).nth(2)).toHaveValue(String(DEFAULT_TIMEOUT_SETTINGS.factoryResetTimeoutSecs));
		await expect(inputs(page).nth(3)).toHaveValue(String(DEFAULT_TIMEOUT_SETTINGS.firmwareUpdateTimeoutSecs));
	});

	test('save failure shows error message', async ({ page }) => {
		await page.route('**/settings', async (route: any) => {
			if (route.request().method() === 'GET') {
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(DEFAULT_TIMEOUT_SETTINGS),
				});
			} else if (route.request().method() === 'POST') {
				await route.fulfill({ status: 500, contentType: 'text/plain', body: 'internal error' });
			} else {
				await route.continue();
			}
		});

		await setupAndLogin(page);
		await navigateToSettings(page);

		await page.getByRole('button', { name: 'Save' }).click();

		await expect(page.getByText('internal error')).toBeVisible({ timeout: 5000 });
	});
});
