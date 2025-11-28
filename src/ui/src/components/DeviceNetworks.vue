<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import NetworkSettings from "../components/NetworkSettings.vue"
import { useCore } from "../composables/useCore"

const { viewModel, initialize, subscribeToChannels } = useCore()

const tab = ref(0)

const networkStatus = computed(() => viewModel.network_status)

onMounted(async () => {
	await initialize()
	subscribeToChannels()
})
</script>

<template>
  <div class="flex flex-col gap-y-4 flex-wrap">
    <div class="flex border-b gap-x-4 items-center">
      <div class="text-h4 text-secondary">Network</div>
    </div>
    <div class="d-flex flex-row">
      <v-tabs v-model="tab" color="primary" direction="vertical">
        <v-tab v-for="networkAdapter in networkStatus?.network_status" :text="networkAdapter.name"
          :value="networkAdapter.name"></v-tab>
      </v-tabs>
      <v-window v-model="tab" class="w[20vw]" direction="vertical">
        <v-window-item v-for="networkAdapter in networkStatus?.network_status" :value="networkAdapter.name">
          <NetworkSettings :networkAdapter="networkAdapter" />
        </v-window-item>
      </v-window>
    </div>
  </div>
</template>