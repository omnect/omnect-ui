<script setup lang="ts">
import { useCore } from "../../composables/useCore"
import { useAsyncAction } from "../../composables/useAsyncAction"

const { reloadNetwork } = useCore()
const { loading, execute } = useAsyncAction()

const handleReloadNetwork = () => execute(reloadNetwork)
</script>

<template>
    <div class="flex flex-col gap-y-4 items-start">
        <div class="text-h4 text-secondary border-b w-100">Commands</div>
        <v-btn :prepend-icon="'mdi-refresh'" variant="text" :loading="loading" :disabled="loading"
            @click="handleReloadNetwork">
            Restart network
        </v-btn>
    </div>
</template>