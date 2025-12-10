<script setup lang="ts">
import { computed, ref, watch } from "vue"
import NetworkSettings from "./NetworkSettings.vue"
import { useCore } from "../../composables/useCore"
import { useCoreInitialization } from "../../composables/useCoreInitialization"

const { viewModel, networkFormReset } = useCore()

useCoreInitialization()

const tab = ref<string | null>(null)
const showUnsavedChangesDialog = ref(false)
const pendingTab = ref<string | null>(null)

const networkStatus = computed(() => viewModel.network_status)

// Determine if an adapter is the current connection by comparing browser hostname with adapter IPs
const isCurrentConnection = (adapter: { readonly ipv4?: { readonly addrs?: readonly { readonly addr: string }[] } }) => {
  const hostname = window.location.hostname
  if (!adapter.ipv4?.addrs) return false
  return adapter.ipv4.addrs.some(ip => ip.addr === hostname)
}

// Watch for tab changes and check for unsaved changes
watch(tab, (newTab, oldTab) => {
  if (newTab === oldTab) return

  // Check if there are unsaved changes
  if (viewModel.network_form_dirty && oldTab !== null) {
    // Block the tab change and show confirmation dialog
    showUnsavedChangesDialog.value = true
    pendingTab.value = newTab as string
    // Revert tab back to old tab
    tab.value = oldTab
  }
})

const confirmTabChange = () => {
  if (pendingTab.value !== null) {
    // User confirmed, discard changes and switch tabs
    const currentAdapter = viewModel.network_form_state?.type === 'editing'
      ? (viewModel.network_form_state as any).adapter_name
      : null

    if (currentAdapter) {
      networkFormReset(currentAdapter)
    }

    // Now switch to the pending tab
    tab.value = pendingTab.value
    pendingTab.value = null
  }
  showUnsavedChangesDialog.value = false
}

const cancelTabChange = () => {
  // User cancelled, stay on current tab
  pendingTab.value = null
  showUnsavedChangesDialog.value = false
}
</script>

<template>
  <div class="flex flex-col gap-y-4 flex-wrap">
    <div class="flex border-b gap-x-4 items-center">
      <div class="text-h4 text-secondary">Network</div>
    </div>
    <div class="d-flex flex-row">
      <v-tabs v-model="tab" color="primary" direction="vertical">
        <v-tab v-for="networkAdapter in networkStatus?.network_status" :text="networkAdapter.name"
          :value="networkAdapter.name"
          :class="{ 'current-connection': isCurrentConnection(networkAdapter) }"></v-tab>
      </v-tabs>
      <v-window v-model="tab" class="w[20vw]" direction="vertical">
        <v-window-item v-for="networkAdapter in networkStatus?.network_status" :value="networkAdapter.name">
          <NetworkSettings :networkAdapter="networkAdapter" :isCurrentConnection="isCurrentConnection(networkAdapter)" />
        </v-window-item>
      </v-window>
    </div>

    <!-- Unsaved changes confirmation dialog -->
    <v-dialog v-model="showUnsavedChangesDialog" max-width="500">
      <v-card>
        <v-card-title class="text-h5">Unsaved Changes</v-card-title>
        <v-card-text>
          You have unsaved changes. Do you want to discard them and switch to another network adapter?
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="primary" text @click="cancelTabChange">Cancel</v-btn>
          <v-btn color="error" text @click="confirmTabChange">Discard Changes</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
.current-connection {
  background-color: rgba(var(--color-primary-rgb), 0.15);
}
</style>