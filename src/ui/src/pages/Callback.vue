<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import { handleRedirectCallback } from "../auth/auth-service"
import { validatePortalToken } from "../auth/validate-portal-token"
import OmnectLogo from "../components/branding/OmnectLogo.vue"

const router = useRouter()
const loading = ref(false)
const errorMsg = ref("")

onMounted(async () => {
	try {
		await handleRedirectCallback()
		loading.value = true
		const valid = await validatePortalToken()
		if (valid) {
			router.replace("/set-password")
		} else {
			errorMsg.value = "You are not authorized."
		}
		loading.value = false
	} catch (e) {
		errorMsg.value = "An error occurred while checking permissions. Please try again."
	}
})
</script>

<template>
	<v-sheet class="mx-auto pa-12 pb-8 m-t-16 flex flex-col gap-y-16 items-center" border elevation="0" max-width="448"
		rounded="lg">
		<OmnectLogo></OmnectLogo>
		<h1>Checking permissions</h1>
		<p v-if="loading">Loading...</p>
		<p class="text-error font-bold font-size-5" v-else>{{ errorMsg }}</p>
	</v-sheet>
</template>