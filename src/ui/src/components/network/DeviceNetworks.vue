<script setup lang="ts">
import { computed, ref } from "vue"
import NetworkSettings from "./NetworkSettings.vue"
import { useCore } from "../../composables/useCore"
import { useCoreInitialization } from "../../composables/useCoreInitialization"

const { viewModel } = useCore()

useCoreInitialization()

const tab = ref(null)

const networkStatus = computed(() => viewModel.network_status)

// Determine if an adapter is the current connection by comparing browser hostname with adapter IPs
const isCurrentConnection = (adapter: { readonly ipv4?: { readonly addrs?: readonly { readonly addr: string }[] } }) => {
  const hostname = window.location.hostname
  if (!adapter.ipv4?.addrs) return false
  return adapter.ipv4.addrs.some(ip => ip.addr === hostname)
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
  </div>
</template>

<style scoped>
.current-connection {
  background-color: rgba(var(--color-primary-rgb), 0.15);
}
</style>