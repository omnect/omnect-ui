<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import OmnectLogo from "../components/OmnectLogo.vue"
import { useCore } from "../composables/useCore"
import { useSnackbar } from "../composables/useSnackbar"

const router = useRouter()
const { viewModel, setPassword, initialize, subscribeToChannels } = useCore()
const { showError } = useSnackbar()
const password = ref<string>("")
const repeatPassword = ref<string>("")
const visible = ref(false)
const errorMsg = ref("")

const handleSubmit = async (): Promise<void> => {
	errorMsg.value = ""
	if (password.value !== repeatPassword.value) {
		errorMsg.value = "Passwords do not match."
	} else {
		await setPassword(password.value)

		// Wait for Core to process the request
		await new Promise(resolve => setTimeout(resolve, 100))

		// Check viewModel state from Core
		if (viewModel.error_message) {
			errorMsg.value = viewModel.error_message
			showError(errorMsg.value)
		} else if (viewModel.success_message) {
			await initialize()
			subscribeToChannels()
			await router.push("/")
		}
	}
}

onMounted(async () => {
	// Initialize Core for this page
	await initialize()
})
</script>

<template>
	<v-sheet class="mx-auto pa-12 pb-8 m-t-16 flex flex-col gap-y-16" border elevation="0" max-width="448" rounded="lg">
		<OmnectLogo></OmnectLogo>
		<h1>Set Password</h1>
		<v-form @submit.prevent @submit="handleSubmit">
			<v-text-field label="Password" :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
				:type="visible ? 'text' : 'password'" density="compact" placeholder="Enter your password"
				prepend-inner-icon="mdi-lock-outline" variant="outlined" @click:append-inner="visible = !visible"
				v-model="password" autocomplete="new-password"></v-text-field>
			<v-text-field label="Repeat password" :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
				:type="visible ? 'text' : 'password'" density="compact" placeholder="Repeat your password"
				prepend-inner-icon="mdi-lock-outline" variant="outlined" @click:append-inner="visible = !visible"
				v-model="repeatPassword" autocomplete="new-password"></v-text-field>
			<p style="color: rgb(var(--v-theme-error))">{{ errorMsg }}</p>
			<v-btn class="mb-8" color="secondary" size="large" variant="text" type="submit" block>
				Set password
			</v-btn>
		</v-form>
	</v-sheet>
</template>
