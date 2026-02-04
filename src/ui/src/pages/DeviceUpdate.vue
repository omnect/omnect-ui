<script setup lang="ts">
import { computed } from "vue"
import UpdateFileUpload from "../components/update/UpdateFileUpload.vue"
import UpdateInfo from "../components/update/UpdateInfo.vue"
import { useCore } from "../composables/useCore"
import { useCoreInitialization } from "../composables/useCoreInitialization"
import { useMessageWatchers } from "../composables/useMessageWatchers"

const { viewModel, loadUpdate } = useCore()

useCoreInitialization()
useMessageWatchers()

const currentVersion = computed(() => viewModel.systemInfo?.os?.version)

// Use viewModel.isLoading to track the load update request
// The Core sets isLoading=true when LoadUpdate is dispatched and false when response is received
const loadUpdateFetching = computed(() => viewModel.isLoading)

const loadUpdateData = (filename?: string) => {
	// filename is passed from file upload, but not from reload button
	// The backend uses a fixed path regardless of the filename
	loadUpdate(filename ?? "")
}
</script>

<template>
	<v-sheet :border="true" rounded class="m-20">
		<v-row class="m-8">
			<v-col sm="12" xl="6">
				<UpdateFileUpload @file-uploaded="loadUpdateData" />
			</v-col>
			<v-col sm="12" xl="6">
				<UpdateInfo :update-manifest="viewModel.updateManifest" :load-update-fetching="loadUpdateFetching"
					:current-version="currentVersion" @reload-update-info="loadUpdateData" />
			</v-col>
		</v-row>
	</v-sheet>
</template>