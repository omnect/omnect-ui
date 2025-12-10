<script setup lang="ts">
import DeviceNetworks from "../components/network/DeviceNetworks.vue"
import { useCoreInitialization } from "../composables/useCoreInitialization"

useCoreInitialization()
</script>

<template>
    <v-sheet :border="true" rounded class="m-20">
        <div class="flex flex-col gap-y-16 m-8">
            <DeviceNetworks></DeviceNetworks>
        </div>
    </v-sheet>
</template>