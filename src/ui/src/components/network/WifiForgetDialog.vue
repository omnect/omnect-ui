<script setup lang="ts">
import { useCore } from "../../composables/useCore"

const { wifiForgetNetwork } = useCore()

const props = defineProps<{
  modelValue: boolean
  ssid: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const forget = async () => {
  await wifiForgetNetwork(props.ssid)
  emit('update:modelValue', false)
}

const cancel = () => {
  emit('update:modelValue', false)
}
</script>

<template>
  <v-dialog :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)" max-width="450">
    <v-card>
      <v-card-title class="text-h5">Forget Network</v-card-title>
      <v-card-text>
        Are you sure you want to forget the network <strong>{{ ssid }}</strong>?
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="primary" variant="text" @click="cancel">Cancel</v-btn>
        <v-btn color="error" variant="flat" @click="forget" data-cy="wifi-forget-confirm-button">
          Forget
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
