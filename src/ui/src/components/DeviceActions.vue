<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import DialogContent from "../components/DialogContent.vue"
import { useCore } from "../composables/useCore"
import { useSnackbar } from "../composables/useSnackbar"

const { viewModel, initialize, subscribeToChannels, reboot, factoryReset } = useCore()
const { showSuccess, showError } = useSnackbar()
const selectedFactoryResetKeys = ref<string[]>([])
const factoryResetDialog = ref(false)
const rebootDialog = ref(false)
const loading = ref(false)

const factoryResetKeys = computed(() => viewModel.factory_reset)

const emit = defineEmits<{
	(event: "rebootInProgress"): void
	(event: "factoryResetInProgress"): void
}>()

const handleReboot = async () => {
	loading.value = true
	await reboot()

	// Wait for Core to process the request
	await new Promise(resolve => setTimeout(resolve, 100))
	loading.value = false

	// Check viewModel state from Core
	if (viewModel.error_message) {
		showError(viewModel.error_message)
	} else if (viewModel.success_message) {
		emit("rebootInProgress")
		rebootDialog.value = false
		showSuccess(viewModel.success_message)
	}
}

const handleFactoryReset = async () => {
	loading.value = true
	await factoryReset("1", selectedFactoryResetKeys.value)

	// Wait for Core to process the request
	await new Promise(resolve => setTimeout(resolve, 100))
	loading.value = false

	// Check viewModel state from Core
	if (viewModel.error_message) {
		showError(viewModel.error_message)
	} else if (viewModel.success_message) {
		emit("factoryResetInProgress")
		factoryResetDialog.value = false
		showSuccess(viewModel.success_message)
	}
}

onMounted(async () => {
	await initialize()
	subscribeToChannels()
})
</script>

<template>
	<div class="flex flex-col gap-y-4 items-start">
		<div class="text-h4 text-secondary border-b w-100">Commands</div>
		<v-btn :prepend-icon="'mdi-restart'" variant="text">
			Reboot
			<v-dialog v-model="rebootDialog" activator="parent" max-width="340" :no-click-animation="true" persistent
				@keydown.esc="rebootDialog = false">
				<DialogContent title="Reboot device" dialog-type="default" :show-close="true"
					@close="rebootDialog = false">
					<div class="flex flex-col gap-2 mb-8">
						Do you really want to restart the device?
					</div>
					<div class="flex justify-end -mr-4 mt-4">
						<v-btn variant="text" color="warning" :loading="loading" :disabled="loading"
							@click="handleReboot">Reboot</v-btn>
						<v-btn variant="text" color="primary" @click="rebootDialog = false">Cancel</v-btn>
					</div>
				</DialogContent>
			</v-dialog>
		</v-btn>
		<v-btn :prepend-icon="'mdi-undo-variant'" variant="text">
			Factory Reset
			<v-dialog v-model="factoryResetDialog" activator="parent" max-width="340" :no-click-animation="true"
				persistent @keydown.esc="factoryResetDialog = false">
				<DialogContent title="Factory reset" dialog-type="default" :show-close="true"
					@close="factoryResetDialog = false">
					<div class="flex flex-col gap-2 mb-8">
						<v-checkbox-btn v-for="(option, index) in factoryResetKeys?.keys" :label="option"
							v-model="selectedFactoryResetKeys" :value="option" :key="index"></v-checkbox-btn>
					</div>
					<div class="flex justify-end -mr-4 mt-4">
						<v-btn variant="text" color="error" :loading="loading" :disabled="loading"
							@click="handleFactoryReset">Reset</v-btn>
						<v-btn variant="text" color="primary" @click="factoryResetDialog = false">Cancel</v-btn>
					</div>
				</DialogContent>
			</v-dialog>
		</v-btn>
	</div>
</template>

<style scoped></style>