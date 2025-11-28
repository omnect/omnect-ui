<script setup lang="ts">
import { ref } from "vue"
import { useSnackbar } from "../composables/useSnackbar"
import { useCore } from "../composables/useCore"

const { showSuccess, showError } = useSnackbar()
const { viewModel, reloadNetwork } = useCore()

const loading = ref(false)

const handleReloadNetwork = async () => {
	loading.value = true
	await reloadNetwork()

	// Wait for Core to process the request
	await new Promise(resolve => setTimeout(resolve, 100))
	loading.value = false

	// Check viewModel state from Core
	if (viewModel.error_message) {
		showError(viewModel.error_message)
	} else if (viewModel.success_message) {
		showSuccess(viewModel.success_message)
	}
}
</script>

<template>
    <div class="flex flex-col gap-y-4 items-start">
        <div class="text-h4 text-secondary border-b w-100">Commands</div>
        <v-btn :prepend-icon="'mdi-refresh'" variant="text" :loading="loading" :disabled="loading"
            @click="handleReloadNetwork">
            Restart network
        </v-btn>
    </div>
</template>