import { createGlobalState } from "@vueuse/core"
import { reactive } from "vue"

export const useOverlaySpinner = createGlobalState(() => {
	const overlaySpinnerState = reactive({
		overlay: false,
		title: "",
		text: ""
	})

	return { overlaySpinnerState }
})
