<script setup lang="ts">
import { useFetch } from "@vueuse/core"
import { onMounted, ref } from "vue"
import UpdateFileUpload from "../components/UpdateFileUpload.vue"
import UpdateInfo from "../components/UpdateInfo.vue"
import { useCentrifuge } from "../composables/useCentrifugo"
import { useOverlaySpinner } from "../composables/useOverlaySpinner"
import { useSnackbar } from "../composables/useSnackbar"
import { CentrifugeSubscriptionType } from "../enums/centrifuge-subscription-type.enum"
import router from "../plugins/router"
import type { SystemInfo } from "../types"
import type { UpdateValidationStatus } from "../types/update-validation-status"

const { overlaySpinnerState, reset } = useOverlaySpinner()
const { showError } = useSnackbar()
const { history, subscribe } = useCentrifuge()
const currentVersion = ref<string>()
const loadUpdatePayload = ref({
	update_file_path: ""
})

const {
	onFetchError: onLoadUpdateError,
	error: loadUpdateError,
	statusCode: loadUpdateStatusCode,
	execute: loadUpdate,
	isFetching: loadUpdateFetching,
	response,
	data
} = useFetch("update/load", { immediate: false }).post(loadUpdatePayload).json()

onLoadUpdateError(async () => {
	if (loadUpdateStatusCode.value === 401) {
		router.push("/login")
	} else {
		showError(`Uploading file failed: ${(await response.value?.text()) ?? loadUpdateError.value}`)
	}
})

const loadUpdateData = (filename: string) => {
	loadUpdatePayload.value = {
		update_file_path: filename
	}
	loadUpdate(false)
}

onMounted(() => {
	history((data: SystemInfo) => {
		currentVersion.value = data.os.version
	}, CentrifugeSubscriptionType.SystemInfo)
	subscribe((data: UpdateValidationStatus) => {
		if (overlaySpinnerState.isUpdateRunning && (data.status === "Succeeded" || data.status === "Recovered")) {
			reset()
		}
	}, CentrifugeSubscriptionType.UpdateStatus)
})
</script>

<template>
	<v-sheet :border="true" rounded class="m-20">
		<v-row class="m-8">
			<v-col sm="12" md="6">
				<UpdateFileUpload @file-uploaded="loadUpdateData" />
			</v-col>
			<v-col sm="12" md="6">
				<UpdateInfo :update-manifest="data" :load-update-fetching="loadUpdateFetching"
					:current-version="currentVersion" @reload-update-info="loadUpdate(false)" />
			</v-col>
		</v-row>
	</v-sheet>
</template>