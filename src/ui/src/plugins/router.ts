import { createRouter, createWebHistory } from "vue-router"
import { login } from "../auth/auth-service"
import { validatePortalToken } from "../auth/validate-portal-token"
import { useCore } from "../composables/useCore"
import Callback from "../pages/Callback.vue"
import DeviceOverview from "../pages/DeviceOverview.vue"
import DeviceUpdate from "../pages/DeviceUpdate.vue"
import Login from "../pages/Login.vue"
import Network from "../pages/Network.vue"
import Settings from "../pages/Settings.vue"
import SetPassword from "../pages/SetPassword.vue"
import UpdatePassword from "../pages/UpdatePassword.vue"

const routes = [
	{ path: "/", component: DeviceOverview, meta: { text: "Device", icon: "mdi-monitor", requiresAuth: true, showMenu: true } },
	{ path: "/network", component: Network, meta: { text: "Network", icon: "mdi-lan", requiresAuth: true, showMenu: true } },
	{ path: "/update", component: DeviceUpdate, meta: { text: "Update", icon: "mdi-update", requiresAuth: true, showMenu: true } },
	{ path: "/settings", component: Settings, meta: { text: "Settings", icon: "mdi-cog", requiresAuth: true, showMenu: true } },
	{ path: "/login", component: Login, meta: { showMenu: false, guestOnly: true, inlineErrors: true } },
	{ path: "/set-password", component: SetPassword, meta: { requiresPortalAuth: true, showMenu: false, inlineErrors: true } },
	{ path: "/update-password", component: UpdatePassword, meta: { requiresAuth: true, showMenu: true, inlineErrors: true } },
	{ path: "/auth-callback", component: Callback, meta: { showMenu: false } },
	{ path: "/:pathMatch(.*)*", redirect: "/" }
]

const router = createRouter({
	history: createWebHistory(),
	routes
})

router.beforeEach(async (to) => {
	const { viewModel, initialize, isInitialized } = useCore()

	// Ensure core is initialized before checking any auth state
	if (!isInitialized.value) {
		await initialize()
	}

	// Redirect authenticated users away from guest-only and portal-auth routes
	if ((to.meta.guestOnly || to.meta.requiresPortalAuth) && viewModel.isAuthenticated) {
		return "/"
	}

	if (to.meta.requiresPortalAuth) {
		// If a password already exists, the set-password flow is not needed.
		// Redirect to regular login before triggering a Keycloak round-trip.
		try {
			const res = await fetch("/require-set-password")
			const passwordRequired = await res.json() as boolean
			if (!passwordRequired) return "/login"
		} catch {
			// Network failure during the check: fall through to portal auth.
		}

		// Validate the portal token against the backend to establish the server-side
		// session flag (portal_validated). Normally Callback.vue does this after the
		// OIDC redirect, but after a factory reset the OIDC user persists in
		// localStorage while the backend session is fresh — so Callback.vue is
		// bypassed and the flag is never set.
		const valid = await validatePortalToken()
		if (!valid) {
			await login()
			return false
		}
	}

	if (to.meta.requiresAuth) {
		// Rely on the Core's authentication state as the single source of truth
		if (!viewModel.isAuthenticated) {
			return "/login"
		}
	}
})

export default router
