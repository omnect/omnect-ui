<script setup lang="ts">
import type { DeviceInfoData } from "../types"

const props = defineProps<{
  deviceInfo: DeviceInfoData,
}>()


</script>

<template>
  <v-list lines="one" variant="flat">
    <v-list-item title="Online" :subtitle="props.deviceInfo.online.toString()"></v-list-item>
    <v-list-item title="OS version" :subtitle="props.deviceInfo.systemInfo?.os?.version"></v-list-item>
    <v-list-item title="OS name" :subtitle="props.deviceInfo.systemInfo?.os?.name"></v-list-item>
    <v-list-item title="Wait online timeout"
      :subtitle="`${props.deviceInfo.timeouts?.wait_online_timeout?.secs}s`"></v-list-item>
    <v-list-item title="Omnect device service version"
      :subtitle="props.deviceInfo.systemInfo?.omnect_device_service_version"></v-list-item>
    <v-list-item title="Azure SDK version" :subtitle="props.deviceInfo.systemInfo?.azure_sdk_version"></v-list-item>
    <v-list-item title="Boot time"
      :subtitle="props.deviceInfo.systemInfo?.boot_time ? new Date(props.deviceInfo.systemInfo?.boot_time).toLocaleString() : ''"></v-list-item>
    <v-list-item title="Factory reset status" :subtitle="props.deviceInfo.factoryResetStatus"></v-list-item>
  </v-list>
</template>

<style scoped>
/* Add any specific styles for DeviceInfo.vue here */
</style>