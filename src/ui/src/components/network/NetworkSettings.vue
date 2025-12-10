<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue"
import { useSnackbar } from "../../composables/useSnackbar"
import { useCore } from "../../composables/useCore"
import { useClipboard } from "../../composables/useClipboard"
import { useIPValidation } from "../../composables/useIPValidation"
import type { DeviceNetwork } from "../../types"

const { showError } = useSnackbar()
const { viewModel, setNetworkConfig, networkFormReset, networkFormUpdate, networkFormStartEdit } = useCore()
const { copy } = useClipboard()
const { isValidIp: validateIp, parseNetmask } = useIPValidation()

const props = defineProps<{
    networkAdapter: DeviceNetwork
}>()

const ipAddress = ref(props.networkAdapter?.ipv4?.addrs[0]?.addr || "")
const dns = ref(props.networkAdapter?.ipv4?.dns?.join("\n") || "")
const gateways = ref(props.networkAdapter?.ipv4?.gateways?.join("\n") || "")
const addressAssignment = ref(props.networkAdapter?.ipv4?.addrs[0]?.dhcp ? "dhcp" : "static")
const netmask = ref(props.networkAdapter?.ipv4?.addrs[0]?.prefix_len || 24)

// Track if this is the initial mount to skip the first form watcher trigger
const isInitialMount = ref(true)

// Initialize form editing state in Core when component mounts
networkFormStartEdit(props.networkAdapter.name)

// Helper to send current form data to Core for dirty flag tracking
const sendFormUpdateToCore = () => {
    const formData = {
        name: props.networkAdapter.name,
        ip_address: ipAddress.value,
        dhcp: addressAssignment.value === "dhcp",
        prefix_len: netmask.value,
        dns: dns.value.split("\n").filter(d => d.trim()),
        gateways: gateways.value.split("\n").filter(g => g.trim())
    }
    networkFormUpdate(JSON.stringify(formData))
}

// Watch form fields and notify Core when they change
// Use flush: 'post' to ensure watcher runs after all DOM updates
watch([ipAddress, dns, gateways, addressAssignment, netmask], () => {
    console.log('[NetworkSettings] Form watcher fired, isInitialMount:', isInitialMount.value, 'isSubmitting:', isSubmitting.value, 'isSyncing:', isSyncingFromWebSocket.value, 'dhcp:', addressAssignment.value)

    // Skip on initial mount - the first WebSocket sync will set correct values
    if (isInitialMount.value) {
        console.log('[NetworkSettings] Skipping form update (initial mount)')
        isInitialMount.value = false
        return
    }

    // Don't update dirty flag during submit or WebSocket sync
    if (!isSubmitting.value && !isSyncingFromWebSocket.value) {
        console.log('[NetworkSettings] Sending form update to Core')
        sendFormUpdateToCore()
    } else {
        console.log('[NetworkSettings] Skipping form update (submitting or syncing)')
    }
}, { flush: 'post' })

// Watch for prop changes from WebSocket updates and sync local state
// Only sync if user is not currently editing (no pending changes)
watch(() => props.networkAdapter, (newAdapter) => {
    if (!newAdapter) return

    // Don't overwrite user's unsaved changes
    if (isSubmitting.value) {
        console.log('[NetworkSettings] WebSocket watcher: Skipping sync during submit')
        return
    }

    // Don't overwrite user's unsaved changes (check dirty flag from Core)
    if (viewModel.network_form_dirty === true) {
        console.log('[NetworkSettings] WebSocket watcher: Skipping sync - user has unsaved changes')
        return
    }

    console.log('[NetworkSettings] WebSocket watcher: Syncing form with WebSocket update:', {
        name: newAdapter.name,
        dhcp: newAdapter.ipv4?.addrs[0]?.dhcp
    })

    // Set flag to prevent form watchers from firing during sync
    console.log('[NetworkSettings] WebSocket watcher: Setting isSyncingFromWebSocket = true')
    isSyncingFromWebSocket.value = true

    ipAddress.value = newAdapter.ipv4?.addrs[0]?.addr || ""
    dns.value = newAdapter.ipv4?.dns?.join("\n") || ""
    gateways.value = newAdapter.ipv4?.gateways?.join("\n") || ""
    addressAssignment.value = newAdapter.ipv4?.addrs[0]?.dhcp ? "dhcp" : "static"
    netmask.value = newAdapter.ipv4?.addrs[0]?.prefix_len || 24

    // Clear flag after Vue finishes all reactive updates AND all post-flush watchers
    // Need double nextTick: first for reactive updates, second for post-flush watchers
    nextTick(() => {
        nextTick(() => {
            console.log('[NetworkSettings] WebSocket watcher: Clearing isSyncingFromWebSocket in double nextTick')
            isSyncingFromWebSocket.value = false
        })
    })
}, { deep: true })

const isDHCP = computed(() => addressAssignment.value === "dhcp")
const isSubmitting = ref(false)
const isSyncingFromWebSocket = ref(false)
const isServerAddr = computed(() => props.networkAdapter?.ipv4?.addrs[0]?.addr === location.hostname)
const ipChanged = computed(() => props.networkAdapter?.ipv4?.addrs[0]?.addr !== ipAddress.value)

const restoreSettings = () => {
    // Reset Core state (clears dirty flag and NetworkFormState)
    networkFormReset(props.networkAdapter.name)

    // Reset local form state
    ipAddress.value = props.networkAdapter?.ipv4?.addrs[0]?.addr || ""
    addressAssignment.value = props.networkAdapter?.ipv4?.addrs[0]?.dhcp ? "dhcp" : "static"
    netmask.value = props.networkAdapter?.ipv4?.addrs[0]?.prefix_len || 24
    dns.value = props.networkAdapter?.ipv4?.dns?.join("\n") || ""
    gateways.value = props.networkAdapter?.ipv4?.gateways?.join("\n") || ""
}

const setNetMask = (mask: string) => {
    const prefixLen = parseNetmask(mask)
    if (prefixLen === null) {
        return "Invalid netmask"
    }
    netmask.value = prefixLen
}

watch(
	() => viewModel.error_message,
	(newMessage) => {
		if (newMessage) {
			showError(newMessage)
			isSubmitting.value = false
		}
	}
)

watch(
	() => viewModel.success_message,
	(newMessage) => {
		if (newMessage) {
			isSubmitting.value = false
		}
	}
)

const submit = async () => {
    console.log('NetworkSettings submit called')
    isSubmitting.value = true

    const config = JSON.stringify({
        isServerAddr: isServerAddr.value,
        ipChanged: ipChanged.value,
        name: props.networkAdapter.name,
        dhcp: isDHCP.value,
        ip: ipAddress.value ?? null,
        previousIp: props.networkAdapter.ipv4?.addrs[0]?.addr,
        netmask: netmask.value ?? null,
        gateway: gateways.value.split("\n").filter(g => g.trim()) ?? [],
        dns: dns.value.split("\n").filter(d => d.trim()) ?? []
    })
    console.log('NetworkSettings config:', config)

    await setNetworkConfig(config)
}
</script>

<template>
    <div>
        <v-form @submit.prevent="submit" class="flex flex-col gap-y-4 ml-4">
            <v-chip size="large" class="ma-2" label
                :color="props.networkAdapter.online ? 'light-green-darken-2' : 'red-darken-2'">
                {{ props.networkAdapter.online ? "Online" : "Offline" }}
            </v-chip>
            <v-radio-group v-model="addressAssignment" inline>
                <v-radio label="DHCP" value="dhcp"></v-radio>
                <v-radio label="Static" value="static"></v-radio>
            </v-radio-group>
            <v-text-field :readonly="isDHCP" v-model="ipAddress" label="IP Address" :rules="[validateIp]" outlined
                append-inner-icon="mdi-content-copy" @click:append-inner="copy(`${ipAddress}/${netmask}`)">
                <template #append-inner>
                    <v-btn :disabled="isDHCP" size="large" append-icon="mdi-menu-down" variant="text" density="compact"
                        slim class="m-0">
                        /{{ netmask }}
                        <v-menu activator="parent">
                            <v-list>
                                <v-list-item v-for="(item, index) in ['/8', '/16', '/24', '/32']" :key="index"
                                    :value="index">
                                    <v-list-item-title @click="setNetMask(item)">{{ item }}</v-list-item-title>
                                </v-list-item>
                            </v-list>
                        </v-menu>
                    </v-btn>
                </template>
            </v-text-field>
            <v-text-field label="MAC Address" variant="outlined" readonly v-model="props.networkAdapter.mac"
                append-inner-icon="mdi-content-copy"
                @click:append-inner="copy(props.networkAdapter.mac)"></v-text-field>
            <v-textarea :readonly="isDHCP" v-model="gateways" label="Gateways" variant="outlined" rows="3" no-resize
                append-inner-icon="mdi-content-copy" @click:append-inner="copy(ipAddress)"></v-textarea>
            <v-textarea v-model="dns" label="DNS" variant="outlined" rows="3" no-resize
                append-inner-icon="mdi-content-copy" @click:append-inner="copy(ipAddress)"></v-textarea>
            <div class="flex flex-row gap-x-4">
                <v-btn color="secondary" type="submit" variant="text" :loading="isSubmitting">
                    Save
                </v-btn>
                <v-btn :disabled="isSubmitting" type="reset" variant="text" @click.prevent="restoreSettings">
                    Reset
                </v-btn>
            </div>
        </v-form>
    </div>
</template>

<style lang="css">
.v-field:has(input[type="text"]:read-only),
.v-field:has(textarea:read-only) {
    background-color: #f5f5f5 !important;
}
</style>
