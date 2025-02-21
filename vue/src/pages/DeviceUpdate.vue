<script setup lang="ts">
import { useFetch } from "@vueuse/core"
import UpdateFileUpload from "../components/UpdateFileUpload.vue"
import UpdateInfo from "../components/UpdateInfo.vue"
import { useSnackbar } from "../composables/useSnackbar"
import router from "../plugins/router"

const { snackbarState } = useSnackbar()
const {
	onFetchError: onLoadUpdateError,
	error: loadUpdateError,
	statusCode: loadUpdateStatusCode,
	execute: loadUpdate,
	isFetching: loadUpdateFetching,
	response,
	data
} = useFetch("update/load", { immediate: false }).post().json()

onLoadUpdateError(async () => {
	if (loadUpdateStatusCode.value === 401) {
		router.push("/login")
	} else {
		showError(`Uploading file failed: ${(await response.value?.text()) ?? loadUpdateError.value}`)
	}
})

const showError = (errorMsg: string) => {
	snackbarState.msg = errorMsg
	snackbarState.color = "error"
	snackbarState.timeout = -1
	snackbarState.snackbar = true
}
</script>

<template>
    <v-sheet :border="true" rounded class="m-20">
		<v-row class="m-8">
			<v-col sm="12" md="6">
				<UpdateFileUpload @file-uploaded="loadUpdate(false)" />
			</v-col>
			<v-col sm="12" md="6">
				<UpdateInfo :update-manifest="data" :load-update-fetching="loadUpdateFetching" @reload-update-info="loadUpdate(false)" />
			</v-col>
		</v-row>
    </v-sheet>
</template>