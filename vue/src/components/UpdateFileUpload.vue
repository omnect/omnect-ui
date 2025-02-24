<script setup lang="ts">
import { useFetch } from "@vueuse/core"
import { ref } from "vue"
import { useSnackbar } from "../composables/useSnackbar"
import router from "../plugins/router"

const { snackbarState } = useSnackbar()

const emit = defineEmits<(e: "fileUploaded", filename: string) => void>()

const updateFile = ref<File | undefined>(undefined)

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
		updateFile.value = undefined
		formData.delete("file")
	}
})

onUploadSuccess(() => {
	if (updateFile.value) {
		emit("fileUploaded", updateFile.value.name)
		updateFile.value = undefined
		formData.delete("file")
	}
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
	await upload(false)
}

const showError = (errorMsg: string) => {
	snackbarState.msg = errorMsg
	snackbarState.color = "error"
	snackbarState.timeout = -1
	snackbarState.snackbar = true
}
</script>

<template>
    <v-form @submit.prevent="uploadFile" enctype="multipart/form-data">
        <v-file-upload icon="mdi-file-upload" v-model="updateFile" clearable density="comfortable" :disabled="uploadFetching"></v-file-upload>
        <v-btn type="submit" prepend-icon="mdi-file-upload-outline" variant="text" :loading="uploadFetching" :disabled="!updateFile" class="mt-4">Upload</v-btn>
    </v-form>
</template>