<script setup lang="ts">
import { onMounted } from "vue"
import { useRouter } from "vue-router"
import { getUser, handleRedirectCallback } from "../auth/auth-service"

const router = useRouter()

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
			}
		} else {
			router.replace("/")
		}
	} catch (e) {
		router.replace("/")
	}
})
</script>

<template>
	<v-sheet :border="true" rounded class="m-20">
		<h1>Callback</h1>
		<p>Loading...</p>
	</v-sheet>
</template>