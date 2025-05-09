import { InMemoryWebStorage, UserManager, WebStorageStateStore } from "oidc-client-ts"

const oidcConfig = {
	authority: "https://keycloak.omnect.conplement.cloud/realms/cp-dev",
	client_id: "omnect-ui",
	redirect_uri: `https://${window.location.hostname}:${window.location.port}/auth-callback`,
	response_type: "code",
	scope: "openid profile email",
	post_logout_redirect_uri: `https://${window.location.hostname}:${window.location.port}/`,
	userStore: new WebStorageStateStore({ store: new InMemoryWebStorage() })
}

const userManager = new UserManager(oidcConfig)

export default userManager
