<script setup lang="ts">
import { ref, watch } from "vue"
import { useSnackbar } from "../../composables/useSnackbar"
import { useCore } from "../../composables/useCore"

const { showSuccess, showError } = useSnackbar()
const { viewModel, reloadNetwork } = useCore()

const loading = ref(false)

watch(
	() => viewModel.success_message,
	(newMessage) => {
		if (newMessage) {
			showSuccess(newMessage)
			loading.value = false
		}
	}
)

watch(
	() => viewModel.error_message,
	(newMessage) => {
		if (newMessage) {
			showError(newMessage)
			loading.value = false
		}
	}
)

const handleReloadNetwork = async () => {
	loading.value = true
	await reloadNetwork()
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