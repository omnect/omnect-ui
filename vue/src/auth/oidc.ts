import { InMemoryWebStorage, UserManager, WebStorageStateStore } from "oidc-client-ts"

const oidcConfig = {
	authority: import.meta.env.VITE_KEYCLOAK_AUTH,
	client_id: "omnect-ui",
	redirect_uri: `https://${window.location.hostname}:${window.location.port}/auth-callback`,
	response_type: "code",
	scope: "openid profile email",
	post_logout_redirect_uri: `https://${window.location.hostname}:${window.location.port}/`,
	userStore: new WebStorageStateStore({ store: new InMemoryWebStorage() })
}

const userManager = new UserManager(oidcConfig)

export default userManager
