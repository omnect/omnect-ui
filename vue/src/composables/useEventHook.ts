export function useEventHook<TParams extends unknown[] = unknown[]>() {
	const fns: Array<(...params: TParams) => void> = []
	function on(fn: (...params: TParams) => void) {
		fns.push(fn)
		return {
			off: () => off(fn)
		}
	}

	function off(fn: (...params: TParams) => void) {
		const index = fns.indexOf(fn)
		if (index !== -1) {
			fns.splice(index, 1)
		}
	}

	function trigger(...params: TParams) {
		for (const fn of fns) {
			fn(...params)
		}
	}

	return {
		on,
		off,
		trigger
	}
}
