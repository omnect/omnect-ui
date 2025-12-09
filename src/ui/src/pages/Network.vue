<script setup lang="ts">
import DeviceNetworks from "../components/network/DeviceNetworks.vue"
import NetworkActions from "../components/network/NetworkActions.vue"
import { useCoreInitialization } from "../composables/useCoreInitialization"

useCoreInitialization()
</script>

<template>
    <v-sheet :border="true" rounded class="m-20">
        <div class="grid grid-cols-[1fr_minmax(200px,auto)] gap-8 gap-x-16 m-8">
            <div class="flex flex-col gap-y-16">
                <DeviceNetworks></DeviceNetworks>
            </div>
            <NetworkActions></NetworkActions>
        </div>
    </v-sheet>
</template>