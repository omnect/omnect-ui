<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue"
import DialogContent from "../components/DialogContent.vue"
import { useCore } from "../composables/useCore"
import { useSnackbar } from "../composables/useSnackbar"

const { viewModel, initialize, reboot, factoryReset } = useCore()
const { showSuccess, showError } = useSnackbar()
const selectedFactoryResetKeys = ref<string[]>([])
const factoryResetDialog = ref(false)
const rebootDialog = ref(false)
const loading = ref(false)

const factoryResetKeys = computed(() => viewModel.factory_reset)

watch(
	() => viewModel.success_message,
	(newMessage) => {
		if (newMessage) {
			if (rebootDialog.value) {
				rebootDialog.value = false
			}
			if (factoryResetDialog.value) {
				factoryResetDialog.value = false
			}
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

const handleReboot = async () => {
	loading.value = true
	await reboot()
}

const handleFactoryReset = async () => {
	loading.value = true
	await factoryReset("1", selectedFactoryResetKeys.value)
}

onMounted(async () => {
	await initialize()
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
						<div v-if="factoryResetKeys?.keys && factoryResetKeys.keys.length > 0">
							<v-checkbox-btn v-for="(option, index) in factoryResetKeys.keys" :label="option"
								v-model="selectedFactoryResetKeys" :value="option" :key="index"></v-checkbox-btn>
						</div>
						<div v-else class="text-grey">
							No preserve options available
						</div>
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