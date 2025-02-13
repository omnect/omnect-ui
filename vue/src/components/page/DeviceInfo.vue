<script setup lang="ts">
import { type Ref, onMounted, ref } from "vue"
import { useCentrifuge } from "../../composables/useCentrifugo"
import { CentrifugeSubscriptionType } from "../../enums/centrifuge-subscription-type.enum"
import { type FactoryResetStatus } from "../../types/factory-reset-status"
import { type OnlineStatus } from "../../types/online-status"
import { type SystemInfo } from "../../types/system-info"
import { type Timeouts } from "../../types/timeouts"

const { subscribe, history } = useCentrifuge()

const online = ref(false)
const systemInfo: Ref<SystemInfo | undefined> = ref(undefined)
const timeouts: Ref<Timeouts | undefined> = ref(undefined)
const factoryResetStatus: Ref<string> = ref("")

const updateOnlineStatus = (data: OnlineStatus) => {
	online.value = data.iothub
}

const updateSystemInfo = (data: SystemInfo) => {
	systemInfo.value = data
}

const updateTimeouts = (data: Timeouts) => {
	timeouts.value = data
}

const updateFactoryResetStatus = (data: FactoryResetStatus) => {
	factoryResetStatus.value = data.factory_reset_status
}

onMounted(() => {
	history(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)
	history(updateSystemInfo, CentrifugeSubscriptionType.SystemInfo)
	history(updateTimeouts, CentrifugeSubscriptionType.Timeouts)
	history(updateFactoryResetStatus, CentrifugeSubscriptionType.FactoryResetStatus)

	subscribe(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)
	subscribe(updateSystemInfo, CentrifugeSubscriptionType.SystemInfo)
	subscribe(updateTimeouts, CentrifugeSubscriptionType.Timeouts)
	subscribe(updateFactoryResetStatus, CentrifugeSubscriptionType.FactoryResetStatus)
})
</script>

<template>
    <h1>Device info</h1>
	<p><b>Online:</b> {{ online }}</p>
	<p><b>OS version:</b> {{ systemInfo?.os?.version }}</p>
	<p><b>OS name:</b> {{ systemInfo?.os?.name }}</p>
	<p><b>Wait online timeout:</b> {{ timeouts?.wait_online_timeout?.secs }} s</p>
	<p><b>Omnect device service version:</b> {{ systemInfo?.omnect_device_service_version }}</p>
	<p><b>Azure SDK version:</b> {{ systemInfo?.azure_sdk_version }}</p>
	<p><b>Boot time:</b> {{ systemInfo?.boot_time ? new Date(systemInfo?.boot_time).toLocaleString() : "" }}</p>
	<p><b>Factory reset status:</b> {{ factoryResetStatus }}</p>
</template>