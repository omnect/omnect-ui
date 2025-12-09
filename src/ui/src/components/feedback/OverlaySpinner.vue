<script setup lang="ts">
const props = defineProps<{
	overlay: boolean
	title: string
	text?: string
	timedOut: boolean
	progress?: number
}>()

const refresh = () => {
	window.location.reload()
}
</script>

<template>
	<v-overlay :persistent="true" :model-value="props.overlay" :no-click-animation="true"
		class="align-center justify-center">
		<div id="overlay" class="flex flex-col items-center">
			<v-sheet class="flex flex-col gap-y-8 items-center p-8" :rounded="'lg'">
				<div class="text-h4 text-center">{{ props.title }}</div>
				<v-progress-circular color="secondary" :indeterminate="props.progress === undefined"
					:model-value="props.progress" size="100" width="5">
					<template v-slot:default>
						<span v-if="props.progress !== undefined" class="text-h6">{{ props.progress }}%</span>
					</template>
				</v-progress-circular>
				<p class="text-h6 m-t-4">{{ props.text }}</p>
				<v-btn v-if="props.timedOut" text="Refresh" @click="refresh" />
			</v-sheet>
		</div>
	</v-overlay>
</template>