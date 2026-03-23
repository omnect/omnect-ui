import { test, expect } from '@playwright/test'
import { setupAndLogin, mockHealthcheck } from './fixtures/test-setup'
import {
  mockConfig,
  mockLoginSuccess,
  mockRequireSetPassword,
  mockTokenRefresh,
} from './fixtures/mock-api'

test.describe('security: route guards reject unauthenticated access', () => {
  const protectedRoutes = ['/', '/network', '/update', '/settings', '/update-password']

  for (const route of protectedRoutes) {
    test(`redirects ${route} to /login without session`, async ({ page }) => {
      await mockConfig(page)
      await mockRequireSetPassword(page)
      await mockTokenRefresh(page, 401)

      await page.goto(route)

      await expect(page).toHaveURL(/\/login(\?.*)?$/)
      await expect(page.getByPlaceholder(/enter your password/i)).toBeVisible()
    })
  }
})

test.describe('security: subscription lifecycle across logout', () => {
  test('WebSocket reconnects after logout and re-login', async ({ page }) => {
    // Mock logout endpoint
    await page.route('**/logout', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({}),
      })
    })

    // First login — WebSocket subscribes to channels
    await setupAndLogin(page)
    await expect(page.getByText('Common Info')).toBeVisible()

    // Logout — should disconnect WebSocket and clear subscriptions
    await page.locator('[data-cy="user-menu"]').click()
    await page.getByRole('button', { name: /logout/i }).click()
    await expect(page.getByPlaceholder(/enter your password/i)).toBeVisible()

    // Re-login — WebSocket should re-subscribe cleanly
    await mockHealthcheck(page)
    await page.getByPlaceholder(/enter your password/i).fill('password')
    await page.getByRole('button', { name: /log in/i }).click()

    // Dashboard should render, proving WebSocket subscriptions were re-established
    await expect(page.getByText('Common Info')).toBeVisible()
  })
})
