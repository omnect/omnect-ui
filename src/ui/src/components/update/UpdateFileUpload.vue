<script setup lang="ts">
import axios, { AxiosError } from "axios"
import { onMounted, ref, watch } from "vue"
import { useCore } from "../../composables/useCore"
import { useSnackbar } from "../../composables/useSnackbar"
import router from "../../plugins/router"

const { showError } = useSnackbar()
const { viewModel } = useCore()
const emit = defineEmits<(e: "fileUploaded", filename: string) => void>()

const updateFile = ref<File>()
const progressPercentage = ref<number | undefined>(0)
const uploadFetching = ref(false)

watch(updateFile, () => {
	progressPercentage.value = 0
})

watch(
	() => viewModel.device_operation_state,
	(state) => {
		if (state.type === 'reconnection_successful' && state.operation === 'Update') {
			updateFile.value = undefined
		}
	},
	{ deep: true }
)

const uploadFile = async () => {
	if (!viewModel.is_authenticated) {
		router.push("/login")
		return
	}

	if (!updateFile.value) {
		showError("Select an update file.")
		return
	}

	if (updateFile.value.type !== "application/x-tar") {
		showError("Wrong file type. Only tar archives are allowed.")
		return
	}

	const formData = new FormData()
	formData.append("file", updateFile.value as File)

	uploadFetching.value = true

	try {
		const res = await axios.post("update/file", formData, {
			withCredentials: true,
			onUploadProgress({ progress }) {
				progressPercentage.value = progress ? Math.ceil(progress * 100) : 0
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
	} catch (err) {
		showError(`Uploading file failed: ${err as AxiosError}`)
	}

	formData.delete("file")
	uploadFetching.value = false
}
</script>

<template>
	<v-form @submit.prevent="uploadFile" enctype="multipart/form-data">
		<v-file-upload icon="mdi-file-upload" v-model="updateFile" clearable density="default"
			:disabled="uploadFetching">
			<template #item="{ file, props }">
				<v-file-upload-item v-bind="props">
					<template #title>
						<div class="flex justify-between">
							<div>{{ file.name }}</div>
							<div v-if="uploadFetching || progressPercentage === 100">{{ progressPercentage }}%</div>
						</div>
					</template>
					<template #subtitle>
						<v-progress-linear v-if="uploadFetching || progressPercentage === 100" class="mt-1"
							:model-value="progressPercentage" :striped="uploadFetching"
							:color="progressPercentage === 100 ? 'success' : 'secondary'"
							:height="10"></v-progress-linear>
					</template>
				</v-file-upload-item>
			</template>
		</v-file-upload>
		<v-btn type="submit" prepend-icon="mdi-file-upload-outline" variant="text"
			:disabled="!updateFile || uploadFetching" class="mt-4">Upload</v-btn>
	</v-form>
</template>