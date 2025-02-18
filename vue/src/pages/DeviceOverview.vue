<script setup lang="ts">
import { type Ref, computed, onMounted, ref } from "vue"
import DeviceInfo from "../components/DeviceInfo.vue"
import DeviceActions from "../components/DeviceActions.vue"
import { useCentrifuge } from "../composables/useCentrifugo"
import { CentrifugeSubscriptionType } from "../enums/centrifuge-subscription-type.enum"
import type { DeviceInfoData, FactoryResetStatus, NetworkStatus, OnlineStatus, SystemInfo, Timeouts } from "../types"
const { subscribe, history, onConnected } = useCentrifuge()

const online = ref(false)
const systemInfo: Ref<SystemInfo | undefined> = ref(undefined)
const timeouts: Ref<Timeouts | undefined> = ref(undefined)
const factoryResetStatus: Ref<string> = ref("")
const networkStatus: Ref<NetworkStatus | undefined> = ref(undefined)
const isResetting = ref(false)
const isRebooting = ref(false)

const deviceInfo: Ref<DeviceInfoData> = computed(() => ({
	systemInfo: systemInfo.value,
	timeouts: timeouts.value,
	factoryResetStatus: factoryResetStatus.value,
	online: online.value
}))

onConnected(() => {
	isResetting.value = false
	isRebooting.value = false
})

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

const updateNetworkStatus = (data: NetworkStatus) => {
	networkStatus.value = data
}

const ipList = (i: number) => {
	const list: any[] = []
	if (!networkStatus.value?.network_status[i]?.ipv4?.addrs || networkStatus.value?.network_status[i]?.ipv4?.addrs?.length === 0) return
	const networkState = networkStatus.value.network_status[i]
	const ipv4 = networkState.ipv4

	list.push({ type: "subheader", title: "MAC" })
	list.push({ title: networkState.mac })
	list.push({ type: "subheader", title: "IPv4" })
	for (const network of ipv4.addrs) {
		list.push({ title: `${network.addr}/${network.prefix_len} (${network.dhcp ? "DHCP" : "Static"})` })
	}
	list.push({ type: "subheader", title: "DNS" })
	for (const dns of ipv4.dns) {
		list.push({ title: dns })
	}
	list.push({ type: "subheader", title: "Gateways" })
	for (const gateway of ipv4.gateways) {
		list.push({ title: gateway })
	}
	return list
}



onMounted(() => {
	history(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)
	history(updateSystemInfo, CentrifugeSubscriptionType.SystemInfo)
	history(updateTimeouts, CentrifugeSubscriptionType.Timeouts)
	history(updateFactoryResetStatus, CentrifugeSubscriptionType.FactoryResetStatus)
	history(updateNetworkStatus, CentrifugeSubscriptionType.NetworkStatus)

	subscribe(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)
	subscribe(updateSystemInfo, CentrifugeSubscriptionType.SystemInfo)
	subscribe(updateTimeouts, CentrifugeSubscriptionType.Timeouts)
	subscribe(updateFactoryResetStatus, CentrifugeSubscriptionType.FactoryResetStatus)
	subscribe(updateNetworkStatus, CentrifugeSubscriptionType.NetworkStatus)
})
</script>

<template>
	<v-overlay :persistent="true" :model-value="isResetting || isRebooting" z-index="1000"
		class="align-center justify-center">
		<div id="overlay" class="flex flex-col items-center">
			<v-card>
				<v-card-title v-if="isResetting">Device is resetting</v-card-title>
				<v-card-title v-else-if="isRebooting">Device is rebooting</v-card-title>
				<v-card-text class="flex flex-col items-center">
					<v-progress-circular color="secondary" indeterminate size="100" width="5"></v-progress-circular>
					<p v-if="isResetting" class="m-t-4">Please have some patience, the resetting may take some time.</p>
				</v-card-text>
			</v-card>
		</div>
	</v-overlay>

	<v-row>
		<v-col cols="12" md="6">
			<h1 class=""></h1>
			<DeviceInfo :deviceInfo="deviceInfo" />
		</v-col>
		<v-col cols="12" md="6">
			<h1>Network</h1>
			<v-row>
				<v-col cols="12" md="2" v-for="(network, index) of networkStatus?.network_status">
					<v-card :title="network.name">
						<template #prepend>
							<div :class="network.online ? 'online' : 'offline'"></div>
						</template>
						<v-card-text>
							<v-list lines="one" :items="ipList(index)"></v-list>
						</v-card-text>
					</v-card>
				</v-col>
			</v-row>
		</v-col>
		<DeviceActions @reboot-in-progess="isResetting = true" @factory-reset-in-progress="isResetting = true">
		</DeviceActions>
	</v-row>
</template>

<style scoped>
.online,
.offline {
	width: 15px;
	height: 15px;
	border-radius: 15px;
}

.online {
	background-color: rgb(var(--v-theme-success));
}

.offline {
	background-color: rgb(var(--v-theme-error));
}
</style>