<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useCore } from "../../composables/useCore"
import type { WifiStateType } from "../../composables/useCore"
import WifiConnectDialog from "./WifiConnectDialog.vue"
import WifiForgetDialog from "./WifiForgetDialog.vue"

const { viewModel, wifiScan, wifiDisconnect, wifiGetStatus, wifiGetSavedNetworks } = useCore()

// Fetch current status and saved networks when panel first mounts
// (init-time fetches fail because auth token is not yet available)
onMounted(() => {
  if (viewModel.wifiState.type === 'ready') {
    wifiGetStatus()
    wifiGetSavedNetworks()
  }
})

const connectDialogOpen = ref(false)
const connectDialogSsid = ref("")
const forgetDialogOpen = ref(false)
const forgetDialogSsid = ref("")

const wifi = computed(() => {
  if (viewModel.wifiState.type !== 'ready') return null
  return viewModel.wifiState
})

const signalIcon = (rssi: number): string => {
  if (rssi >= -50) return 'mdi-wifi-strength-4'
  if (rssi >= -60) return 'mdi-wifi-strength-3'
  if (rssi >= -70) return 'mdi-wifi-strength-2'
  return 'mdi-wifi-strength-1'
}

const openConnectDialog = (ssid: string) => {
  connectDialogSsid.value = ssid
  connectDialogOpen.value = true
}

const openForgetDialog = (ssid: string) => {
  forgetDialogSsid.value = ssid
  forgetDialogOpen.value = true
}
</script>

<template>
  <div v-if="wifi" class="mt-8">
    <!-- Connection Status -->
    <div class="text-h5 text-secondary font-weight-bold border-b pb-2 mb-4">WiFi Connection</div>

    <div class="mb-6">
      <div v-if="wifi.status.state.type === 'connected'" class="d-flex align-center gap-4">
        <span class="text-body-1">{{ wifi.status.ssid }}</span>
        <v-btn
          color="error" variant="flat" size="small"
          @click="wifiDisconnect()"
          data-cy="wifi-disconnect-button"
        >
          Disconnect
        </v-btn>
      </div>

      <div v-else-if="wifi.status.state.type === 'connecting'" class="d-flex align-center gap-2">
        <v-progress-circular indeterminate size="16" width="2" />
        <span class="text-body-2">Connecting to {{ wifi.status.ssid }}...</span>
      </div>

      <v-alert v-else-if="wifi.status.state.type === 'failed'" type="error" variant="tonal" density="compact">
        {{ wifi.status.state.message }}
      </v-alert>

      <div v-else class="text-body-2 text-medium-emphasis">
        Not connected
      </div>
    </div>

    <!-- Available Networks -->
    <div class="text-h5 text-secondary font-weight-bold border-b pb-2 mb-4">Available Networks</div>

    <div class="mb-2">
      <v-btn color="primary" variant="flat" size="small"
        :loading="wifi.scanState.type === 'scanning'"
        @click="wifiScan()"
        data-cy="wifi-scan-button"
      >
        Scan
      </v-btn>
    </div>

    <v-alert v-if="wifi.scanState.type === 'error'" type="error" variant="tonal" density="compact" class="mb-4">
      {{ wifi.scanState.message }}
    </v-alert>

    <v-list v-if="wifi.scanResults.length > 0" lines="one" density="compact" class="mb-6">
      <v-list-item
        v-for="network in wifi.scanResults"
        :key="network.ssid"
        @click="openConnectDialog(network.ssid)"
        class="cursor-pointer"
        :data-cy="`wifi-network-${network.ssid}`"
      >
        <template #prepend>
          <v-icon :icon="signalIcon(network.rssi)" size="small" class="mr-3"></v-icon>
        </template>
        <v-list-item-title>{{ network.ssid }}</v-list-item-title>
        <template #append>
          <span class="text-caption text-medium-emphasis">Ch {{ network.channel }}</span>
        </template>
      </v-list-item>
    </v-list>

    <div v-else-if="wifi.scanState.type === 'finished'" class="text-body-2 text-medium-emphasis mb-6">
      No networks found.
    </div>

    <!-- Saved Networks -->
    <div class="text-h5 text-secondary font-weight-bold border-b pb-2 mb-4">Saved Networks</div>

    <v-list v-if="wifi.savedNetworks.length > 0" lines="one" density="compact">
      <v-list-item
        v-for="network in wifi.savedNetworks"
        :key="network.ssid"
        :data-cy="`wifi-saved-${network.ssid}`"
      >
        <v-list-item-title>
          {{ network.ssid }}
          <v-chip v-if="network.flags.includes('CURRENT')" size="x-small" label color="primary" class="ml-2">Current</v-chip>
        </v-list-item-title>
        <template #append>
          <v-btn icon size="x-small" variant="text" @click="openForgetDialog(network.ssid)" :data-cy="`wifi-forget-${network.ssid}`">
            <v-icon icon="mdi-delete-outline" size="small"></v-icon>
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <div v-else class="text-body-2 text-medium-emphasis">
      No saved networks.
    </div>

    <!-- Dialogs -->
    <WifiConnectDialog v-model="connectDialogOpen" :ssid="connectDialogSsid" />
    <WifiForgetDialog v-model="forgetDialogOpen" :ssid="forgetDialogSsid" />
  </div>
</template>
