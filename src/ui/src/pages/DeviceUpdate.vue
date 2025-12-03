<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue"
import UpdateFileUpload from "../components/UpdateFileUpload.vue"
import UpdateInfo from "../components/UpdateInfo.vue"
import { useCore } from "../composables/useCore"
import { useSnackbar } from "../composables/useSnackbar"

const { showError, showSuccess } = useSnackbar()
const { viewModel, initialize, loadUpdate } = useCore()

const loadUpdateFetching = ref(false)

const currentVersion = computed(() => viewModel.system_info?.os?.version)

watch(
	() => viewModel.error_message,
	(newMessage) => {
		if (newMessage) {
			showError(newMessage)
		}
	}
)

watch(
	() => viewModel.success_message,
	(newMessage) => {
		if (newMessage) {
			showSuccess(newMessage)
		}
	}
)

const loadUpdateData = async () => {
	loadUpdateFetching.value = true
	// loadUpdate is called after file upload, no parameter needed
	await loadUpdate("")
	loadUpdateFetching.value = false
}

onMounted(async () => {
	await initialize()
})
</script>

<template>
	<v-sheet :border="true" rounded class="m-20">
		<v-row class="m-8">
			<v-col sm="12" xl="6">
				<UpdateFileUpload @file-uploaded="loadUpdateData" />
			</v-col>
			<v-col sm="12" xl="6">
				<UpdateInfo :update-manifest="viewModel.update_manifest" :load-update-fetching="loadUpdateFetching"
					:current-version="currentVersion" @reload-update-info="loadUpdateData" />
			</v-col>
		</v-row>
	</v-sheet>
</template>