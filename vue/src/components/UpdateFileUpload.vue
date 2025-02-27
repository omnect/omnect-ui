<script setup lang="ts">
import axios from "axios"
import { ref } from "vue"
import { useSnackbar } from "../composables/useSnackbar"
import router from "../plugins/router"

const { snackbarState } = useSnackbar()

const emit = defineEmits<(e: "fileUploaded", filename: string) => void>()

const updateFile = ref<File>()
const progressPercentage = ref<number | undefined>(0)
const uploadFetching = ref(false)

const uploadFile = async () => {
	if (!updateFile.value) {
		showError("Select an update file")
		return
	}

	if (updateFile.value.type !== "application/x-tar") {
		showError("Wrong file type. Only tar archives are allowed")
		return
	}

	const formData = new FormData()
	formData.append("file", updateFile.value as File)

	uploadFetching.value = true

	const res = await axios.post("update/file", formData, {
		onUploadProgress({ progress }) {
			progressPercentage.value = progress ? Math.round(progress * 100) : 0
		},
		responseType: "text"
	})

	if (res.status < 300) {
		emit("fileUploaded", updateFile.value.name)
	} else if (res.status === 401) {
		router.push("/login")
	} else {
		showError(`Uploading file failed: ${res.data}`)
	}

	progressPercentage.value = 0
	formData.delete("file")
	uploadFetching.value = false
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
		<v-file-upload icon="mdi-file-upload" v-model="updateFile" clearable density="default"
			:disabled="uploadFetching">
			<template #item="{ file }">
				<v-file-upload-item>
					<template #title>
						<div class="flex justify-between">
							<div>{{ file.name }}</div>
							<div v-if="uploadFetching || progressPercentage === 100">{{ progressPercentage }}%</div>
						</div>
					</template>
					<template #subtitle>
						<v-progress-linear v-if="uploadFetching || progressPercentage === 100" class="mt-1"
							:model-value="progressPercentage" striped color="secondary"
							:height="10"></v-progress-linear>
					</template>
				</v-file-upload-item>
			</template>
		</v-file-upload>
		<v-btn type="submit" prepend-icon="mdi-file-upload-outline" variant="text" :loading="uploadFetching"
			:disabled="!updateFile" class="mt-4">Upload</v-btn>
	</v-form>
</template>