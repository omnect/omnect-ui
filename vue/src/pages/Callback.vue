<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import { getUser, handleRedirectCallback } from "../auth/auth-service"

const router = useRouter()
const loading = ref(true)
const errorMsg = ref("")

onMounted(async () => {
	try {
		await handleRedirectCallback()
		const user = await getUser()
		if (user) {
			const res = await fetch("token/validate", {
				method: "POST",
				headers: {
					"Content-Type": "plain/text"
				},
				body: user.access_token
			})

			if (res.ok) {
				router.replace("/set-password")
			} else {
				errorMsg.value = "You are not authorized."
			}
		} else {
			errorMsg.value = "You are not authorized."
		}
	} catch (e) {
		errorMsg.value = "An error occurred while checking permissions. Please try again."
	}
})
</script>

<template>
	<v-sheet :border="true" rounded class="m-20">
		<h1>Checking permissions</h1>
		<p v-if="loading">Loading...</p>
		<p v-else>{{ errorMsg }}</p>
	</v-sheet>
</template>