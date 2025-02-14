<script setup lang="ts">
import { type Ref, onMounted, ref } from "vue"
import { useCentrifuge } from "../../composables/useCentrifugo"
import { CentrifugeSubscriptionType } from "../../enums/centrifuge-subscription-type.enum"
import type { FactoryResetKeys } from "../../types/factory-reset-keys"
import type { FactoryResetStatus } from "../../types/factory-reset-status"
import type { NetworkStatus } from "../../types/network-status"
import type { OnlineStatus } from "../../types/online-status"
import type { SystemInfo } from "../../types/system-info"
import type { Timeouts } from "../../types/timeouts"
import DialogContent from "../DialogContent.vue"
import { useRouter } from "vue-router"

const { subscribe, history, onConnected } = useCentrifuge()
const router = useRouter()

const online = ref(false)
const systemInfo: Ref<SystemInfo | undefined> = ref(undefined)
const timeouts: Ref<Timeouts | undefined> = ref(undefined)
const factoryResetStatus: Ref<string> = ref("")
const networkStatus: Ref<NetworkStatus | undefined> = ref(undefined)
const factoryResetKeys: Ref<FactoryResetKeys | undefined> = ref(undefined)
const selectedFactoryResetKeys: Ref<string[]> = ref([])
const factoryResetDialog = ref(false)
const rebootDialog = ref(false)
const loading = ref(false)
const reloadNetworkLoading = ref(false)
const isResetting = ref(false)
const isRebooting = ref(false)
const color = ref("")
const timeout = ref(-1)
const msg = ref("")
const snackbar = ref(false)

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

const updateFactoryResetKeys = (data: FactoryResetKeys) => {
	factoryResetKeys.value = data
}

const showError = (errorMsg: string) => {
	msg.value = errorMsg
	color.value = "error"
	timeout.value = -1
	snackbar.value = true
}

const showSuccess = (successMsg: string) => {
	msg.value = successMsg
	color.value = "success"
	timeout.value = 2000
	snackbar.value = true
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

const rebootDevice = async () => {
	try {
		loading.value = true
		const res = await fetch("reboot", {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			}
		})
		if (res.ok) {
			isRebooting.value = true
			rebootDialog.value = false
		} else if(res.status === 401) {
			router.push("/login")
		} else {
			showError("Rebooting device failed")
		}
	} catch (error) {
		showError("Failed to send reboot request")
	} finally {
		loading.value = false
	}
}

const resetDevice = async () => {
	try {
		loading.value = true
		const res = await fetch("factory-reset", {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			body: JSON.stringify({ preserve: selectedFactoryResetKeys.value })
		})
		if (res.ok) {
			isResetting.value = true
			factoryResetDialog.value = false
		} else if(res.status === 401) {
			router.push("/login")
		} else {
			showError("Resetting device failed")
		}
	} catch (error) {
		showError("Failed to send reset request")
	} finally {
		loading.value = false
	}
}

const reloadNetwork = async () => {
	reloadNetworkLoading.value = true
	try {
		const res = await fetch("reload-network", {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			}
		})
		if (res.ok) {
			showSuccess("Reload network successful")
		} else if(res.status === 401) {
			router.push("/login")
		} else {
			showError("Reload network failed")
		}
	} catch (error) {
		showError("Failed to send reload network request")
	} finally {
		reloadNetworkLoading.value = false
	}
}

onMounted(() => {
	history(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)
	history(updateSystemInfo, CentrifugeSubscriptionType.SystemInfo)
	history(updateTimeouts, CentrifugeSubscriptionType.Timeouts)
	history(updateFactoryResetStatus, CentrifugeSubscriptionType.FactoryResetStatus)
	history(updateNetworkStatus, CentrifugeSubscriptionType.NetworkStatus)
	history(updateFactoryResetKeys, CentrifugeSubscriptionType.FactoryResetKeys)

	subscribe(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)
	subscribe(updateSystemInfo, CentrifugeSubscriptionType.SystemInfo)
	subscribe(updateTimeouts, CentrifugeSubscriptionType.Timeouts)
	subscribe(updateFactoryResetStatus, CentrifugeSubscriptionType.FactoryResetStatus)
	subscribe(updateNetworkStatus, CentrifugeSubscriptionType.NetworkStatus)
	subscribe(updateFactoryResetKeys, CentrifugeSubscriptionType.FactoryResetKeys)
})
</script>

<template>
	<v-overlay :persistent="true" :model-value="isResetting || isRebooting" z-index="1000" class="align-center justify-center">
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
			<h1>Device info</h1>
			<v-list lines="one" variant="flat">
				<v-list-item title="Online" :subtitle="online.toString()"></v-list-item>
				<v-list-item title="OS version" :subtitle="systemInfo?.os?.version"></v-list-item>
				<v-list-item title="OS name" :subtitle="systemInfo?.os?.name"></v-list-item>
				<v-list-item title="Wait online timeout" :subtitle="`${timeouts?.wait_online_timeout?.secs}s`"></v-list-item>
				<v-list-item title="Omnect device service version" :subtitle="systemInfo?.omnect_device_service_version"></v-list-item>
				<v-list-item title="Azure SDK version" :subtitle="systemInfo?.azure_sdk_version"></v-list-item>
				<v-list-item title="Boot time" :subtitle="systemInfo?.boot_time ? new Date(systemInfo?.boot_time).toLocaleString() : ''"></v-list-item>
				<v-list-item title="Factory reset status" :subtitle="factoryResetStatus"></v-list-item>
			</v-list>
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
		<v-col cols="12" md="6">
			<h1>Commands</h1>
			<v-btn-group divided variant="flat" color="secondary">
				<v-btn>
					Reboot
					<v-dialog v-model="rebootDialog" activator="parent" max-width="340" :no-click-animation="true" persistent @keydown.esc="rebootDialog = false">
						<DialogContent title="Reboot device" dialog-type="default" :show-close="true" @close="rebootDialog = false">
							<div class="flex flex-col gap-2 mb-8">
								Do you really want to restart the device?
							</div>
							<div class="flex justify-end -mr-4 mt-4">
								<v-btn variant="text" color="warning" :loading="loading" :disabled="loading" @click="rebootDevice">Reboot</v-btn>
								<v-btn variant="text" color="primary" @click="rebootDialog = false">Cancel</v-btn>
							</div>
						</DialogContent>
					</v-dialog>
				</v-btn>
				<v-btn>
					Factory reset
					<v-dialog v-model="factoryResetDialog" activator="parent" max-width="340" :no-click-animation="true" persistent @keydown.esc="factoryResetDialog = false">
						<DialogContent title="Factory reset" dialog-type="default" :show-close="true" @close="factoryResetDialog = false">
							<div class="flex flex-col gap-2 mb-8">
								<v-checkbox-btn v-for="(option, index) in factoryResetKeys?.keys" :label="option" v-model="selectedFactoryResetKeys"
									:value="option" :key="index"></v-checkbox-btn>
							</div>
							<div class="flex justify-end -mr-4 mt-4">
								<v-btn variant="text" color="error" :loading="loading" :disabled="loading" @click="resetDevice">Reset</v-btn>
								<v-btn variant="text" color="primary" @click="factoryResetDialog = false">Cancel</v-btn>
							</div>
						</DialogContent>
					</v-dialog>
				</v-btn>
				<v-btn :loading="reloadNetworkLoading" @click="reloadNetwork">Reload network</v-btn>
			</v-btn-group>
		</v-col>
	</v-row>
	<v-snackbar v-model="snackbar" :color="color" :timeout="timeout">
		{{ msg }}
		<template #actions>
			<v-btn icon=" mdi-close" @click="snackbar = false"></v-btn>
		</template>
	</v-snackbar>
</template>

<style scoped>
.online, .offline {
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