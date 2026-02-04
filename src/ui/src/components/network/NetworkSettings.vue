<script setup lang="ts">
import { computed, nextTick, ref, watch, type DeepReadonly } from "vue"
import { useSnackbar } from "../../composables/useSnackbar"
import { useCore, NetworkConfigRequest } from "../../composables/useCore"
import type { DeviceNetwork } from "../../composables/useCore"
import { useClipboard } from "../../composables/useClipboard"

const { showError } = useSnackbar()
const { viewModel, setNetworkConfig, networkFormReset, networkFormUpdate } = useCore()
const { copy } = useClipboard()

const props = defineProps<{
    networkAdapter: DeepReadonly<DeviceNetwork>
    isCurrentConnection: boolean
}>()

const cidrToSubnet = (cidr: number): string => {
    const mask = cidr === 0 ? 0 : (~0 << (32 - cidr)) >>> 0;
    return [
        (mask >>> 24) & 0xff,
        (mask >>> 16) & 0xff,
        (mask >>> 8) & 0xff,
        mask & 0xff
    ].join('.');
};

const ipAddress = ref(props.networkAdapter?.ipv4?.addrs[0]?.addr || "")
const dns = ref(props.networkAdapter?.ipv4?.dns?.join("\n") || "")
const gateways = ref(props.networkAdapter?.ipv4?.gateways?.join("\n") || "")
const addressAssignment = ref(props.networkAdapter?.ipv4?.addrs[0]?.dhcp ? "dhcp" : "static")
const subnetMask = ref(cidrToSubnet(props.networkAdapter?.ipv4?.addrs[0]?.prefix_len ?? 24))

// State flags
const isSubmitting = ref(false)
const isSyncingFromCore = ref(false)

const syncLocalFieldsFromCore = (formData: any) => {
    isSyncingFromCore.value = true
    ipAddress.value = formData.ipAddress
    dns.value = formData.dns.join("\n")
    gateways.value = formData.gateways.join("\n")
    addressAssignment.value = formData.dhcp ? "dhcp" : "static"
    subnetMask.value = formData.subnetMask

    // Ensure all reactive updates complete before allowing form updates to be sent back to Core
    nextTick(() => {
        isSyncingFromCore.value = false
    })
}

// Watch for form state changes from Core
watch(() => viewModel.networkFormState, (state) => {
    if (state?.type === 'editing' && state.adapterName === props.networkAdapter.name) {
        // If not dirty, we sync everything from Core (initialization or reset)
        if (!viewModel.networkFormDirty) {
            syncLocalFieldsFromCore(state.formData)
        }
    }
}, { immediate: true })

const sendFormUpdateToCore = () => {
    const formData = {
        name: props.networkAdapter.name,
        ipAddress: ipAddress.value,
        dhcp: addressAssignment.value === "dhcp",
        subnetMask: subnetMask.value,
        dns: dns.value.split("\n").filter(d => d.trim()),
        gateways: gateways.value.split("\n").filter(g => g.trim())
    }
    networkFormUpdate(JSON.stringify(formData))
}

watch([ipAddress, dns, gateways, addressAssignment, subnetMask], () => {
    if (!isSubmitting.value && !isSyncingFromCore.value) {
        sendFormUpdateToCore()
    }
})

// Note: We no longer watch props.networkAdapter directly for form fields.
// Core is the source of truth for the "original" data.

const isDHCP = computed(() => addressAssignment.value === "dhcp")

const isRollbackRequired = computed(() => viewModel.shouldShowRollbackModal)
const enableRollback = ref(true)
const confirmationModalOpen = ref(false)

watch(() => viewModel.shouldShowRollbackModal, (shouldShow) => {
    if (shouldShow) {
        enableRollback.value = viewModel.defaultRollbackEnabled
    }
})

const switchingToDhcp = computed(() => !props.networkAdapter?.ipv4?.addrs[0]?.dhcp && isDHCP.value)

const restoreSettings = () => {
    networkFormReset(props.networkAdapter.name)
}

watch(
	() => viewModel.errorMessage,
	(newMessage) => {
		if (newMessage) {
			showError(newMessage)
			isSubmitting.value = false
		}
	}
)

watch(
	() => viewModel.successMessage,
	(newMessage) => {
		if (newMessage) {
			isSubmitting.value = false
            confirmationModalOpen.value = false
		}
	}
)

watch(
	() => viewModel.networkFormState,
	(newState) => {
		if (isSubmitting.value && newState?.type !== 'submitting') {
			isSubmitting.value = false
		}
	}
)

const submit = async () => {
    if (isRollbackRequired.value) {
        confirmationModalOpen.value = true
    } else {
        await submitNetworkConfig(false)
    }
}

const submitNetworkConfig = async (includeRollback: boolean) => {
    isSubmitting.value = true
    confirmationModalOpen.value = false

    const config = new NetworkConfigRequest(
        props.isCurrentConnection,
        props.networkAdapter.ipv4?.addrs[0]?.addr !== ipAddress.value,
        props.networkAdapter.name,
        isDHCP.value,
        ipAddress.value || null,
        props.networkAdapter.ipv4?.addrs[0]?.addr || null,
        null, // netmask will be determined by Core
        gateways.value.split("\n").filter(g => g.trim()) || [],
        dns.value.split("\n").filter(d => d.trim()) || [],
        includeRollback ? enableRollback.value : null,
        switchingToDhcp.value
    )

    await setNetworkConfig(JSON.stringify(config))
}

const cancelRollbackModal = () => {
    confirmationModalOpen.value = false
}

const errors = computed(() => {
    if (viewModel.networkFormState?.type === 'editing' || viewModel.networkFormState?.type === 'submitting') {
        return (viewModel.networkFormState as any).errors
    }
    return {}
})
</script>

<template>
    <div>
        <!-- Rollback Confirmation Modal -->
        <v-dialog v-model="confirmationModalOpen" max-width="600">
            <v-card>
                <v-card-title class="text-h5">
                    Confirm Network Configuration Change
                </v-card-title>
                <v-card-text>
                    <v-alert type="warning" variant="tonal" class="mb-4">
                        This change will disconnect your current session.
                    </v-alert>
                    <p class="mb-4">
                        <template v-if="switchingToDhcp">
                            You are about to switch to DHCP on the network adapter you're currently connected to.
                            This will likely assign a new IP address and interrupt your connection.
                        </template>
                        <template v-else>
                            You are about to change the IP address of the network adapter you're currently connected to.
                            This will interrupt your connection.
                        </template>
                    </p>
          <v-checkbox
            v-model="enableRollback"
            :label="switchingToDhcp ? 'Enable automatic rollback' : 'Enable automatic rollback (recommended)'"
            hide-details
          >
            <template v-slot:label>
              <strong>{{ switchingToDhcp ? 'Enable automatic rollback' : 'Enable automatic rollback (recommended)' }}</strong>
            </template>
          </v-checkbox>
          <div class="text-caption text-medium-emphasis ml-8">
            <template v-if="switchingToDhcp">
              Not recommended for DHCP: You won't know the new IP address, making it difficult to confirm the change before the 90 second timeout triggers a rollback.
            </template>
            <template v-else>
              If you can't reach the new IP and log in within 90 seconds, the device will automatically restore the previous configuration.
            </template>
          </div>
                </v-card-text>
                <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn color="secondary" variant="text" @click="cancelRollbackModal">
                        Cancel
                    </v-btn>
                    <v-btn color="primary" variant="text" @click="submitNetworkConfig(true)" data-cy="network-confirm-apply-button">
                        Apply Changes
                    </v-btn>
                </v-card-actions>
            </v-card>
        </v-dialog>

        <v-form @submit.prevent="submit" class="ml-4">
            <v-chip size="large" class="ma-2 mb-6" label
                :color="props.networkAdapter.online ? 'light-green-darken-2' : 'red-darken-2'">
                {{ props.networkAdapter.online ? "Online" : "Offline" }}{{ props.isCurrentConnection && props.networkAdapter.online ? " (current connection)" : "" }}
            </v-chip>

            <v-container fluid class="pa-0">
                <v-row>
                    <v-col cols="12" md="6">
                         <v-radio-group v-model="addressAssignment" inline label="Address Assignment" hide-details>
                            <v-radio label="DHCP" value="dhcp"></v-radio>
                            <v-radio label="Static" value="static"></v-radio>
                        </v-radio-group>
                    </v-col>
                    <v-col cols="12" md="6">
                        <v-text-field label="MAC Address" variant="outlined" readonly v-model="props.networkAdapter.mac"
                            append-inner-icon="mdi-content-copy"
                            @click:append-inner="copy(props.networkAdapter.mac)"></v-text-field>
                    </v-col>

                    <v-col cols="12" md="6">
                        <v-text-field :readonly="isDHCP" v-model="ipAddress" label="IP Address" :error-messages="errors?.ipAddress" outlined
                            append-inner-icon="mdi-content-copy" @click:append-inner="copy(ipAddress)"></v-text-field>
                    </v-col>
                    <v-col cols="12" md="6">
                        <v-text-field :readonly="isDHCP" v-model="subnetMask" label="Subnet Mask" :error-messages="errors?.subnetMask" outlined
                            append-inner-icon="mdi-content-copy" @click:append-inner="copy(subnetMask)" hint="e.g. 255.255.255.0"></v-text-field>
                    </v-col>

                    <v-col cols="12" md="6">
                        <v-textarea :readonly="isDHCP" v-model="gateways" label="Gateways" variant="outlined" rows="3" no-resize
                            append-inner-icon="mdi-content-copy" @click:append-inner="copy(gateways)"></v-textarea>
                    </v-col>
                    <v-col cols="12" md="6">
                        <v-textarea v-model="dns" label="DNS" variant="outlined" rows="3" no-resize
                            append-inner-icon="mdi-content-copy" @click:append-inner="copy(dns)"></v-textarea>
                    </v-col>
                </v-row>
            </v-container>

            <div class="sticky-footer bg-surface border-t py-4 d-flex gap-x-4 align-center mt-4">
                <v-btn color="secondary" type="submit" variant="text" :loading="isSubmitting" :disabled="!viewModel.networkFormDirty" data-cy="network-apply-button">
                    Apply Changes
                </v-btn>
                <v-btn :disabled="isSubmitting || !viewModel.networkFormDirty" type="reset" variant="text" @click.prevent="restoreSettings" data-cy="network-discard-button">
                    Discard Changes
                </v-btn>
            </div>
        </v-form>
    </div>
</template>

<style lang="css" scoped>
.v-field:has(input[type="text"]:read-only),
.v-field:has(textarea:read-only) {
    background-color: #f5f5f5 !important;
}

.sticky-footer {
  position: sticky;
  bottom: 0;
  z-index: 10;
  background-color: rgb(var(--v-theme-surface));
  width: 100%;
}
</style>
