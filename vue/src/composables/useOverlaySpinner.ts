import { createGlobalState } from "@vueuse/core"
import { reactive } from "vue"

export const useOverlaySpinner = createGlobalState(() => {
	const overlaySpinnerState = reactive({
		overlay: false,
		title: "",
		text: "",
		isUpdateRunning: false
	})

	const reset = () => {
		overlaySpinnerState.overlay = false
		overlaySpinnerState.text = ""
		overlaySpinnerState.title = ""
		overlaySpinnerState.isUpdateRunning = false
	}

	return { overlaySpinnerState, reset }
})
