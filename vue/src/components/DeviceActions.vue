<script setup lang="ts">
import { useFetch } from '@vueuse/core'
import DialogContent from "../components/DialogContent.vue"
import { useRouter } from 'vue-router'
import { type Ref, ref, computed, onMounted } from 'vue'
import { useCentrifuge } from '../composables/useCentrifugo'
import { CentrifugeSubscriptionType } from '../enums/centrifuge-subscription-type.enum'
import type { FactoryResetKeys } from '../types'
import { useSnackbar } from '../composables/useSnackbar'

const { subscribe, history } = useCentrifuge()
const { snackbarState } = useSnackbar()
const router = useRouter()
const selectedFactoryResetKeys: Ref<string[]> = ref([])
const factoryResetDialog = ref(false)
const rebootDialog = ref(false)
const reloadNetworkLoading = ref(false)
const factoryResetKeys: Ref<FactoryResetKeys | undefined> = ref(undefined)

const emit = defineEmits<{
  (event: 'rebootInProgess'): void,
  (event: 'factoryResetInProgress'): void
}>()

const { onFetchError: onRebootError,
  error: rebootError,
  statusCode: rebootStatusCode,
  onFetchResponse: onRebootSuccess,
  execute: reboot,
  isFetching: rebootFetching } = useFetch("reboot", { immediate: false }).post()

const { onFetchError: onResetError,
  error: resetError,
  statusCode: resetStatusCode,
  onFetchResponse: onResetSuccess,
  execute: reset,
  isFetching: resetFetching } = useFetch("factory-reset", { immediate: false }).post(JSON.stringify({ preserve: selectedFactoryResetKeys.value }))

const { onFetchError: onReloadNetworkError,
  error: reloadNetworkError,
  statusCode: reloadNetworkStatusCode,
  onFetchResponse: onReloadNetworkSuccess,
  execute: reloadNetwork,
  isFetching: reloadNetworkFetching } = useFetch("reload-network", { immediate: false }).post()


const loading = computed(() => rebootFetching.value || resetFetching.value || reloadNetworkFetching.value)

onRebootSuccess(() => {
  console.log("device is rebooting...")
  emit('rebootInProgess')
  rebootDialog.value = false
})

onRebootError(() => {
  if (rebootStatusCode.value === 401) {
    router.push('/login')
  } else {
    showError(`Rebooting device failed: ${JSON.stringify(rebootError.value)}`)
  }
})

onResetSuccess(() => {
  console.log("device is rebooting...")
  emit('factoryResetInProgress')
  factoryResetDialog.value = false
})

onResetError(() => {
  if (resetStatusCode.value === 401) {
    router.push('/login')
  } else {
    showError(`Resetting device failed: ${JSON.stringify(resetError.value)}`)
  }
})

onReloadNetworkSuccess(() => {
  showSuccess("Reload network successful")
})

onReloadNetworkError(() => {
  if (reloadNetworkStatusCode.value === 401) {
    router.push('/login')
  } else {
    showError(`Reloading network failed: ${JSON.stringify(reloadNetworkError.value)}`)
  }
})

const showError = (errorMsg: string) => {
  snackbarState.msg = errorMsg
  snackbarState.color = "error"
  snackbarState.timeout = -1
  snackbarState.snackbar = true
}

const showSuccess = (successMsg: string) => {
  snackbarState.msg = successMsg
  snackbarState.color = "success"
  snackbarState.timeout = 2000
  snackbarState.snackbar = true
}

const updateFactoryResetKeys = (data: FactoryResetKeys) => {
  factoryResetKeys.value = data
}

onMounted(() => {
  subscribe(updateFactoryResetKeys, CentrifugeSubscriptionType.FactoryResetKeys)
  history(updateFactoryResetKeys, CentrifugeSubscriptionType.FactoryResetKeys)
})
</script>

<template>
  <v-col cols="12" md="6">
    <h1>Commands</h1>
    <v-btn-group divided variant="flat" color="secondary">
      <v-btn>
        Reboot
        <v-dialog v-model="rebootDialog" activator="parent" max-width="340" :no-click-animation="true" persistent
          @keydown.esc="rebootDialog = false">
          <DialogContent title="Reboot device" dialog-type="default" :show-close="true" @close="rebootDialog = false">
            <div class="flex flex-col gap-2 mb-8">
              Do you really want to restart the device?
            </div>
            <div class="flex justify-end -mr-4 mt-4">
              <v-btn variant="text" color="warning" :loading="loading" :disabled="loading"
                @click="reboot(false)">Reboot</v-btn>
              <v-btn variant="text" color="primary" @click="rebootDialog = false">Cancel</v-btn>
            </div>
          </DialogContent>
        </v-dialog>
      </v-btn>
      <v-btn>
        Factory reset
        <v-dialog v-model="factoryResetDialog" activator="parent" max-width="340" :no-click-animation="true" persistent
          @keydown.esc="factoryResetDialog = false">
          <DialogContent title="Factory reset" dialog-type="default" :show-close="true"
            @close="factoryResetDialog = false">
            <div class="flex flex-col gap-2 mb-8">
              <v-checkbox-btn v-for="(option, index) in factoryResetKeys?.keys" :label="option"
                v-model="selectedFactoryResetKeys" :value="option" :key="index"></v-checkbox-btn>
            </div>
            <div class="flex justify-end -mr-4 mt-4">
              <v-btn variant="text" color="error" :loading="loading" :disabled="loading"
                @click="reset(false)">Reset</v-btn>
              <v-btn variant="text" color="primary" @click="factoryResetDialog = false">Cancel</v-btn>
            </div>
          </DialogContent>
        </v-dialog>
      </v-btn>
      <v-btn :loading="reloadNetworkLoading" @click="reloadNetwork">Reload network</v-btn>
    </v-btn-group>
  </v-col>
</template>

<style scoped></style>