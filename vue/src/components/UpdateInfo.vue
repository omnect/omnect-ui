<script setup lang="ts">
import { toRef } from "vue"
import type { UpdateManifest } from "../types/update-manifest"
import KeyValuePair from "./ui-components/KeyValuePair.vue"

const props = defineProps<{
	updateManifest: UpdateManifest | undefined
	loadUpdateFetching: boolean
}>()

defineEmits<(event: "reloadUpdateInfo") => void>()

const updateManifest = toRef(props, "updateManifest")
</script>

<template>
    <div class="flex flex-col gap-y-8">
        <div class="flex border-b gap-x-4 items-center">
            <div class="text-h4 text-secondary">Update Info</div>
                <v-btn prepend-icon="mdi-reload" :loading="loadUpdateFetching" @click="$emit('reloadUpdateInfo')"
                variant="text">Load update Info</v-btn>
        </div>
        <dl v-if="updateManifest" class="grid grid-cols-[1fr_3fr] gap-x-64 gap-y-8">
            <KeyValuePair title="Version">{{ updateManifest.update_id.version }}</KeyValuePair>
        </dl>
    </div>
    <v-btn v-if="updateManifest" class="justify-end" prepend-icon="mdi-update" variant="text">Install update</v-btn>
</template>