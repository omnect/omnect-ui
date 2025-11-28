<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import UpdateFileUpload from "../components/UpdateFileUpload.vue"
import UpdateInfo from "../components/UpdateInfo.vue"
import { useCore } from "../composables/useCore"
import { useSnackbar } from "../composables/useSnackbar"

const { showError } = useSnackbar()
const { viewModel, initialize, subscribeToChannels, loadUpdate } = useCore()

const loadUpdateFetching = ref(false)
const data = ref()

const currentVersion = computed(() => viewModel.system_info?.os?.version)

const loadUpdateData = async () => {
	loadUpdateFetching.value = true
	// loadUpdate is called after file upload, no parameter needed
	await loadUpdate("")

	// Wait for Core to process the request
	await new Promise(resolve => setTimeout(resolve, 100))
	loadUpdateFetching.value = false

	// Check viewModel state from Core
	if (viewModel.error_message) {
		showError(viewModel.error_message)
	}
}

onMounted(async () => {
	await initialize()
	subscribeToChannels()
})
</script>

<template>
	<v-sheet :border="true" rounded class="m-20">
		<v-row class="m-8">
			<v-col sm="12" xl="6">
				<UpdateFileUpload @file-uploaded="loadUpdateData" />
			</v-col>
			<v-col sm="12" xl="6">
				<UpdateInfo :update-manifest="data" :load-update-fetching="loadUpdateFetching"
					:current-version="currentVersion" @reload-update-info="loadUpdateData" />
			</v-col>
		</v-row>
	</v-sheet>
</template>