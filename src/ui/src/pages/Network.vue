<script setup lang="ts">
import { ref } from "vue"
import { onBeforeRouteLeave } from "vue-router"
import DeviceNetworks from "../components/network/DeviceNetworks.vue"
import { useCoreInitialization } from "../composables/useCoreInitialization"
import { useCore } from "../composables/useCore"

useCoreInitialization()

const { viewModel, networkFormReset } = useCore()
const showNavigationDialog = ref(false)
let pendingNavigation: (() => void) | null = null

// Navigation guard to prevent leaving page with unsaved changes
onBeforeRouteLeave((_to, _from, next) => {
  console.log('[Network] onBeforeRouteLeave called, dirty flag:', viewModel.network_form_dirty)
  if (viewModel.network_form_dirty) {
    console.log('[Network] Blocking navigation, showing dialog')
    showNavigationDialog.value = true
    pendingNavigation = () => next() // Save the next callback to be called after confirmation
    next(false) // Block navigation
  } else {
    console.log('[Network] Allowing navigation')
    next() // Allow navigation
  }
})

const confirmNavigation = () => {
  // User confirmed, discard changes and navigate
  const currentAdapter = viewModel.network_form_state?.type === 'editing'
    ? (viewModel.network_form_state as any).adapter_name
    : null

  if (currentAdapter) {
    networkFormReset(currentAdapter)
  }

  showNavigationDialog.value = false

  // Trigger pending navigation if it exists
  if (pendingNavigation) {
    pendingNavigation()
    pendingNavigation = null
  }
}

const cancelNavigation = () => {
  // User cancelled, stay on current page
  showNavigationDialog.value = false
  pendingNavigation = null
}
</script>

<template>
    <v-sheet :border="true" rounded class="m-20">
        <div class="flex flex-col gap-y-16 m-8">
            <DeviceNetworks></DeviceNetworks>
        </div>
    </v-sheet>

    <!-- Unsaved changes confirmation dialog (page navigation) -->
    <v-dialog v-model="showNavigationDialog" max-width="500">
      <v-card>
        <v-card-title class="text-h5">Unsaved Changes</v-card-title>
        <v-card-text>
          You have unsaved changes. Do you want to discard them and leave this page?
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="primary" text @click="cancelNavigation">Cancel</v-btn>
          <v-btn color="error" text @click="confirmNavigation">Discard Changes</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
</template>