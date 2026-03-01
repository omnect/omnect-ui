import { getUser, removeUser } from "./auth-service"

/**
 * Validates the OIDC portal token against the backend.
 *
 * On success, the backend sets the `portal_validated` session flag.
 * On failure, clears the stale OIDC user from localStorage.
 *
 * @returns `true` if the token is valid, `false` otherwise
 */
export async function validatePortalToken(): Promise<boolean> {
	const user = await getUser()
	if (!user || user.expired) {
		return false
	}
	try {
		const res = await fetch("/token/validate", {
			method: "POST",
			headers: { "Content-Type": "text/plain" },
			body: user.access_token,
		})
		if (!res.ok) {
			await removeUser()
			return false
		}
		return true
	} catch {
		await removeUser()
		return false
	}
}
