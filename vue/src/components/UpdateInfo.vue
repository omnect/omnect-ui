<script setup lang="ts">
import { useFetch } from "@vueuse/core"
import { toRef } from "vue"
import { useOverlaySpinner } from "../composables/useOverlaySpinner"
import { useSnackbar } from "../composables/useSnackbar"
import router from "../plugins/router"
import type { UpdateManifest } from "../types/update-manifest"
import KeyValuePair from "./ui-components/KeyValuePair.vue"

const { snackbarState } = useSnackbar()
const { overlaySpinnerState } = useOverlaySpinner()

const props = defineProps<{
	updateManifest: UpdateManifest | undefined
	currentVersion: string | undefined
	loadUpdateFetching: boolean
}>()

defineEmits<(event: "reloadUpdateInfo") => void>()

const updateManifest = toRef(props, "updateManifest")

const {
	onFetchError: onRunUpdateError,
	error: runUpdateError,
	statusCode: runUpdateStatusCode,
	execute: runUpdate,
	response
} = useFetch("update/run", { immediate: false }).post()

onRunUpdateError(async () => {
	if (runUpdateStatusCode.value === 401) {
		router.push("/login")
	} else {
		showError(`Running update failed: ${(await response.value?.text()) ?? runUpdateError.value}`)
	}
})

const triggerUpdate = () => {
	runUpdate(false)
	overlaySpinnerState.title = "Installing update"
	overlaySpinnerState.text = "Please have some patience, the update may take some time."
	overlaySpinnerState.overlay = true
}

const showError = (errorMsg: string) => {
	overlaySpinnerState.overlay = false

	snackbarState.msg = errorMsg
	snackbarState.color = "error"
	snackbarState.timeout = -1
	snackbarState.snackbar = true
}
</script>

<template>
	<div class="flex flex-col gap-y-8">
		<div class="flex border-b gap-x-4 items-center">
			<div class="text-h4 text-secondary">Update Info</div>
			<v-btn prepend-icon="mdi-reload" :disabled="!updateManifest" :loading="loadUpdateFetching"
				@click="$emit('reloadUpdateInfo')" variant="text">Load update Info</v-btn>
		</div>
		<dl v-if="updateManifest" class="grid grid-cols-[1fr_3fr] gap-x-64 gap-y-8">
			<KeyValuePair title="Provider">{{ updateManifest.updateId.provider }}</KeyValuePair>
			<KeyValuePair title="Variant">{{ updateManifest.updateId.name }}</KeyValuePair>
			<KeyValuePair title="Current version">{{ props.currentVersion }}</KeyValuePair>
			<KeyValuePair title="Update version">{{ updateManifest.updateId.version }}</KeyValuePair>
			<KeyValuePair title="Manufacturer">{{ updateManifest.compatibility[0].manufacturer }}</KeyValuePair>
			<KeyValuePair title="Model">{{ updateManifest.compatibility[0].model }}</KeyValuePair>
			<KeyValuePair title="Compatibility Id">{{ updateManifest.compatibility[0].compatibilityid }}</KeyValuePair>
			<KeyValuePair title="Created">{{ updateManifest.createdDateTime ? new
				Date(updateManifest.createdDateTime).toLocaleString() : "" }}</KeyValuePair>
		</dl>
	</div>
	<v-btn v-if="updateManifest" class="mt-4" prepend-icon="mdi-update" variant="text" @click="triggerUpdate()">Install
		update</v-btn>
</template>