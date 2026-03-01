<script setup lang="ts">
import { ref, watch } from "vue"
import { useCore } from "../../composables/useCore"

const { wifiConnect, viewModel } = useCore()

const props = defineProps<{
  modelValue: boolean
  ssid: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const password = ref("")
const showPassword = ref(false)

const isConnecting = () => {
  if (viewModel.wifiState.type !== 'ready') return false
  return viewModel.wifiState.status.state.type === 'connecting'
}

const connect = async () => {
  await wifiConnect(props.ssid, password.value)
  emit('update:modelValue', false)
  password.value = ""
}

const cancel = () => {
  emit('update:modelValue', false)
  password.value = ""
}

watch(() => props.modelValue, (open) => {
  if (open) {
    password.value = ""
    showPassword.value = false
  }
})
</script>

<template>
  <v-dialog :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)" max-width="450">
    <v-card>
      <v-card-title class="text-h5">Connect to {{ ssid }}</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="password"
          :type="showPassword ? 'text' : 'password'"
          label="Password"
          :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showPassword = !showPassword"
          @keyup.enter="connect"
          autofocus
          data-cy="wifi-password-input"
        ></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="primary" variant="text" @click="cancel">Cancel</v-btn>
        <v-btn color="primary" variant="flat" @click="connect" :loading="isConnecting()" data-cy="wifi-connect-button">
          Connect
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
