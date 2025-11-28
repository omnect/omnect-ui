<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue"
import { useRouter } from "vue-router"
import OmnectLogo from "../components/OmnectLogo.vue"
import { useCore } from "../composables/useCore"

const { viewModel, login, checkRequiresPasswordSet, initialize } = useCore()
const router = useRouter()

const password = ref("")
const visible = ref(false)
const isCheckingPasswordSetNeeded = ref(false)

// Use viewModel error message instead of local state
const errorMsg = computed(() => viewModel.error_message || "")

// Watch for successful authentication
watch(() => viewModel.is_authenticated, async (isAuthenticated) => {
	if (isAuthenticated) {
		await router.push("/")
	}
})

const doLogin = async (e: Event) => {
	e.preventDefault()
	await login(password.value)
}

onMounted(async () => {
	isCheckingPasswordSetNeeded.value = true

	// Initialize Core and check if password needs to be set
	await initialize()
	await checkRequiresPasswordSet()

	// Wait for the response to be processed
	await new Promise(resolve => setTimeout(resolve, 100))

	if (viewModel.requires_password_set) {
		await router.push("/set-password")
	}

	isCheckingPasswordSetNeeded.value = false
})
</script>

<template>
	<v-sheet class="mx-auto pa-12 pb-8 m-t-16 flex flex-col gap-y-16" border elevation="0" max-width="448" rounded="lg">
		<OmnectLogo></OmnectLogo>
		<v-form v-if="!isCheckingPasswordSetNeeded" @submit.prevent @submit="doLogin">
			<v-text-field label="Password" :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
				:type="visible ? 'text' : 'password'" density="compact" placeholder="Enter your password"
				prepend-inner-icon="mdi-lock-outline" variant="outlined" @click:append-inner="visible = !visible"
				v-model="password" autocomplete="current-password"></v-text-field>
			<p style="color: rgb(var(--v-theme-error))">{{ errorMsg }}</p>
			<v-btn class="mb-8" color="secondary" size="large" variant="text" type="submit" block>
				Log In
			</v-btn>
		</v-form>
	</v-sheet>
</template>