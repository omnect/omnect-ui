<script setup lang="ts">
import { reactive, watch } from "vue"
import { useCore } from "../composables/useCore"
import { useCoreInitialization } from "../composables/useCoreInitialization"
import { useMessageWatchers } from "../composables/useMessageWatchers"

const { viewModel, saveSettings } = useCore()

useCoreInitialization()
useMessageWatchers()

// Local form state — initialised from Core model, editable by user
const form = reactive({
	rebootTimeoutSecs: 0,
	factoryResetTimeoutSecs: 0,
	firmwareUpdateTimeoutSecs: 0,
	networkRollbackTimeoutSecs: 0,
})

// Sync form from Core model once settings are loaded
watch(
	() => viewModel.timeoutSettings,
	(settings) => {
		if (!settings) return
		form.rebootTimeoutSecs = settings.rebootTimeoutSecs
		form.factoryResetTimeoutSecs = settings.factoryResetTimeoutSecs
		form.firmwareUpdateTimeoutSecs = settings.firmwareUpdateTimeoutSecs
		form.networkRollbackTimeoutSecs = settings.networkRollbackTimeoutSecs
	},
	{ immediate: true },
)

const MIN_TIMEOUT_SECS = 30
const MAX_TIMEOUT_SECS = 3600

function resetToDefaults() {
	const defaults = {
		rebootTimeoutSecs: 300,
		factoryResetTimeoutSecs: 600,
		firmwareUpdateTimeoutSecs: 900,
		networkRollbackTimeoutSecs: 90,
	}
	Object.assign(form, defaults)
}

function save() {
	saveSettings({
		rebootTimeoutSecs: form.rebootTimeoutSecs,
		factoryResetTimeoutSecs: form.factoryResetTimeoutSecs,
		firmwareUpdateTimeoutSecs: form.firmwareUpdateTimeoutSecs,
		networkRollbackTimeoutSecs: form.networkRollbackTimeoutSecs,
	})
}
</script>

<template>
	<v-sheet :border="true" rounded class="ma-4">
		<v-row class="ma-4">
			<v-col cols="12">
				<div class="text-h4 text-secondary border-b pb-2 mb-4">Settings</div>
			</v-col>
			<v-col cols="12" md="8" lg="6">
				<div class="text-h5 text-secondary font-weight-bold border-b pb-2 mb-4">Operation Timeouts</div>
				<p class="text-body-2 text-medium-emphasis mb-6">
					How long the UI waits for device operations to complete before reporting failure.
					Changes take effect on the next operation.
				</p>

				<v-row dense>
					<v-col cols="8">
						<span class="text-subtitle-2 text-medium-emphasis">Network config rollback</span>
					</v-col>
					<v-col cols="4">
						<v-text-field
							v-model.number="form.networkRollbackTimeoutSecs"
							type="number"
							variant="outlined"
							density="compact"
							suffix="seconds"
							:min="MIN_TIMEOUT_SECS"
							:max="MAX_TIMEOUT_SECS"
							hide-details="auto"
							:rules="[v => (v >= MIN_TIMEOUT_SECS && v <= MAX_TIMEOUT_SECS) || `${MIN_TIMEOUT_SECS}–${MAX_TIMEOUT_SECS} s`]"
						/>
					</v-col>

					<v-col cols="8">
						<span class="text-subtitle-2 text-medium-emphasis">Reboot</span>
					</v-col>
					<v-col cols="4">
						<v-text-field
							v-model.number="form.rebootTimeoutSecs"
							type="number"
							variant="outlined"
							density="compact"
							suffix="seconds"
							:min="MIN_TIMEOUT_SECS"
							:max="MAX_TIMEOUT_SECS"
							hide-details="auto"
							:rules="[v => (v >= MIN_TIMEOUT_SECS && v <= MAX_TIMEOUT_SECS) || `${MIN_TIMEOUT_SECS}–${MAX_TIMEOUT_SECS} s`]"
						/>
					</v-col>

					<v-col cols="8">
						<span class="text-subtitle-2 text-medium-emphasis">Factory reset</span>
					</v-col>
					<v-col cols="4">
						<v-text-field
							v-model.number="form.factoryResetTimeoutSecs"
							type="number"
							variant="outlined"
							density="compact"
							suffix="seconds"
							:min="MIN_TIMEOUT_SECS"
							:max="MAX_TIMEOUT_SECS"
							hide-details="auto"
							:rules="[v => (v >= MIN_TIMEOUT_SECS && v <= MAX_TIMEOUT_SECS) || `${MIN_TIMEOUT_SECS}–${MAX_TIMEOUT_SECS} s`]"
						/>
					</v-col>

					<v-col cols="8">
						<span class="text-subtitle-2 text-medium-emphasis">Firmware update</span>
					</v-col>
					<v-col cols="4">
						<v-text-field
							v-model.number="form.firmwareUpdateTimeoutSecs"
							type="number"
							variant="outlined"
							density="compact"
							suffix="seconds"
							:min="MIN_TIMEOUT_SECS"
							:max="MAX_TIMEOUT_SECS"
							hide-details="auto"
							:rules="[v => (v >= MIN_TIMEOUT_SECS && v <= MAX_TIMEOUT_SECS) || `${MIN_TIMEOUT_SECS}–${MAX_TIMEOUT_SECS} s`]"
						/>
					</v-col>
				</v-row>

				<div class="d-flex justify-end gap-2 mt-6">
					<v-btn variant="text" color="primary" @click="resetToDefaults">Reset to defaults</v-btn>
					<v-btn variant="flat" color="primary" :loading="viewModel.isLoading" @click="save">Save</v-btn>
				</div>
			</v-col>
		</v-row>
	</v-sheet>
</template>
