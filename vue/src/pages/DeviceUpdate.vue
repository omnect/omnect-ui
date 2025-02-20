<script setup lang="ts">
import { useFetch } from "@vueuse/core"
import { type Ref, ref } from "vue"
import { useSnackbar } from "../composables/useSnackbar"
import router from "../plugins/router"

const { snackbarState } = useSnackbar()

const updateFile: Ref<File | undefined> = ref(undefined)
const formData = new FormData()

const {
	onFetchError: onUploadError,
	error: uploadError,
	statusCode: uploadStatusCode,
	onFetchResponse: onUploadSuccess,
	execute: upload,
	isFetching: uploadFetching,
	response
} = useFetch("update/file", { immediate: false }).post(formData).text()

onUploadError(async () => {
	if (uploadStatusCode.value === 401) {
		router.push("/login")
	} else {
		showError(`Uploading file failed: ${(await response.value?.text()) ?? uploadError.value}`)
	}
})

onUploadSuccess(() => {
	showSuccess("File uploaded")
	updateFile.value = undefined
	formData.delete("file")
})

const uploadFile = async () => {
	if (!updateFile.value) {
		showError("Select an update file")
		return
	}

	if (updateFile.value.type !== "application/x-tar") {
		showError("Wrong file type. Only tar archives are allowed")
		return
	}

	formData.append("file", updateFile.value as File)
	await upload()
}

const showError = (errorMsg: string) => {
	snackbarState.msg = errorMsg
	snackbarState.color = "error"
	snackbarState.timeout = -1
	snackbarState.snackbar = true
}

const showSuccess = (sucessMsg: string) => {
	snackbarState.msg = sucessMsg
	snackbarState.color = "success"
	snackbarState.timeout = 2000
	snackbarState.snackbar = true
}
</script>

<template>
    <v-sheet :border="true" rounded class="m-20">
        <div class="grid grid-cols-[1fr_auto] gap-8 gap-x-16 m-8">
            <v-form @submit.prevent="uploadFile" enctype="multipart/form-data">
                <v-file-upload v-model="updateFile" clearable density="comfortable" :disabled="uploadFetching"></v-file-upload>
                <v-btn type="submit" :loading="uploadFetching" :disabled="!updateFile" class="mt-4">Upload</v-btn>
            </v-form>
        </div>
    </v-sheet>
</template>