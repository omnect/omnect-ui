<script setup lang="ts">
import { ref, watch } from "vue"
import { useRouter } from "vue-router"
import { useSnackbar } from "../composables/useSnackbar"
import { useCore } from "../composables/useCore"

const router = useRouter()
const { showSuccess, showError } = useSnackbar()
const { viewModel, updatePassword } = useCore()
const currentPassword = ref<string>("")
const password = ref<string>("")
const repeatPassword = ref<string>("")
const visible = ref(false)
const errorMsg = ref("")

watch(
	() => viewModel.success_message,
	async (newMessage) => {
		if (newMessage) {
			showSuccess(newMessage)
			await router.push("/login")
		}
	}
)

watch(
	() => viewModel.error_message,
	(newMessage) => {
		if (newMessage) {
			errorMsg.value = newMessage
			showError(errorMsg.value)
		}
	}
)

const handleSubmit = async (): Promise<void> => {
	errorMsg.value = ""
	if (password.value !== repeatPassword.value) {
		errorMsg.value = "Passwords do not match."
	} else {
		await updatePassword(currentPassword.value, password.value)
	}
}
</script>

<template>
	<v-sheet class="mx-auto pa-12 pb-8 m-t-16 flex flex-col gap-y-16" border elevation="0" max-width="448" rounded="lg">
		<h1>Update Password</h1>
		<v-form @submit.prevent @submit="handleSubmit">
			<v-text-field label="Current password" :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
				:type="visible ? 'text' : 'password'" density="compact" placeholder="Enter current your password"
				prepend-inner-icon="mdi-lock-outline" variant="outlined" @click:append-inner="visible = !visible"
				v-model="currentPassword" autocomplete="current-password"></v-text-field>
			<v-text-field label="New password" :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
				:type="visible ? 'text' : 'password'" density="compact" placeholder="Enter your new password"
				prepend-inner-icon="mdi-lock-outline" variant="outlined" @click:append-inner="visible = !visible"
				v-model="password" autocomplete="new-password"></v-text-field>
			<v-text-field label="Repeat new password" :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
				:type="visible ? 'text' : 'password'" density="compact" placeholder="Repeat your new password"
				prepend-inner-icon="mdi-lock-outline" variant="outlined" @click:append-inner="visible = !visible"
				v-model="repeatPassword" autocomplete="new-password"></v-text-field>
			<p style="color: rgb(var(--v-theme-error))">{{ errorMsg }}</p>
			<v-btn class="mb-8" color="secondary" size="large" variant="text" type="submit" block>
				Set new password
			</v-btn>
		</v-form>
	</v-sheet>
</template>
