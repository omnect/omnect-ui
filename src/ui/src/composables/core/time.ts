/**
 * Time capability implementation for Crux Core
 *
 * Handles Timer effects: NotifyAfter, NotifyAt, Clear, and Now.
 * Each pending timer is tracked by TimerId so it can be cancelled via Clear.
 */

// When VITE_RECONNECTION_POLL_INTERVAL_MS is set (E2E test build), cap all crux_time
// notify_after delays to that value so poll cycles complete without real wall-clock delays.
// Production builds leave this undefined and use the actual Core-specified delays.
const TIMER_CAP_MS: number | null = import.meta.env.VITE_RECONNECTION_POLL_INTERVAL_MS
	? Number(import.meta.env.VITE_RECONNECTION_POLL_INTERVAL_MS)
	: null

import { wasmModule } from './state'
import {
	TimeRequest,
	TimeRequestVariantnow,
	TimeRequestVariantnotifyAfter,
	TimeRequestVariantnotifyAt,
	TimeRequestVariantclear,
	TimeResponse,
	TimeResponseVariantnow,
	TimeResponseVariantdurationElapsed,
	TimeResponseVariantinstantArrived,
	TimeResponseVariantcleared,
	TimerId,
	Instant,
} from '../../../../shared_types/generated/typescript/types/shared_types'
import { BincodeSerializer } from '../../../../shared_types/generated/typescript/bincode/mod'

// Effects processor callback - set by effects.ts to avoid circular dependency
let processEffectsCallback: ((effectsBytes: Uint8Array) => Promise<void>) | null = null

export function setEffectsProcessor(callback: (effectsBytes: Uint8Array) => Promise<void>): void {
	processEffectsCallback = callback
}

// Track pending timers by timer ID (BigInt → browser timer handle)
const pendingTimers = new Map<bigint, ReturnType<typeof setTimeout>>()

/**
 * Send a TimeResponse back to Core and process resulting effects
 */
async function resolveTimer(requestId: number, response: TimeResponse): Promise<void> {
	if (!wasmModule.value) return

	const serializer = new BincodeSerializer()
	response.serialize(serializer)
	const responseBytes = serializer.getBytes()
	const newEffectsBytes = wasmModule.value.handle_response(requestId, responseBytes) as Uint8Array
	if (newEffectsBytes.length > 0 && processEffectsCallback) {
		await processEffectsCallback(newEffectsBytes)
	}
}

/**
 * Execute a TimeRequest effect
 */
export async function executeTimeRequest(requestId: number, request: TimeRequest): Promise<void> {
	if (!wasmModule.value) {
		console.warn('[Time Effect] WASM module not loaded')
		return
	}

	if (request instanceof TimeRequestVariantnotifyAfter) {
		const { id, duration } = request
		// duration.nanos is total nanoseconds as BigInt; convert to milliseconds
		const rawDelayMs = Number(duration.nanos / 1_000_000n)
		const delayMs = TIMER_CAP_MS !== null ? Math.min(rawDelayMs, TIMER_CAP_MS) : rawDelayMs
		const handle = setTimeout(async () => {
			pendingTimers.delete(id.value)
			await resolveTimer(requestId, new TimeResponseVariantdurationElapsed(id))
		}, delayMs)
		pendingTimers.set(id.value, handle)
	} else if (request instanceof TimeRequestVariantnotifyAt) {
		const { id, instant } = request
		// instant.seconds + instant.nanos in milliseconds from Unix epoch
		const targetMs = Number(instant.seconds) * 1000 + Math.round(instant.nanos / 1_000_000)
		const delayMs = Math.max(0, targetMs - Date.now())
		const handle = setTimeout(async () => {
			pendingTimers.delete(id.value)
			await resolveTimer(requestId, new TimeResponseVariantinstantArrived(id))
		}, delayMs)
		pendingTimers.set(id.value, handle)
	} else if (request instanceof TimeRequestVariantclear) {
		const { id } = request
		const handle = pendingTimers.get(id.value)
		if (handle !== undefined) {
			clearTimeout(handle)
			pendingTimers.delete(id.value)
		}
		await resolveTimer(requestId, new TimeResponseVariantcleared(id))
	} else if (request instanceof TimeRequestVariantnow) {
		const nowMs = Date.now()
		const seconds = BigInt(Math.floor(nowMs / 1000))
		const nanos = (nowMs % 1000) * 1_000_000
		const instant = new Instant(seconds, nanos)
		await resolveTimer(requestId, new TimeResponseVariantnow(instant))
	} else {
		console.warn('[Time Effect] Unknown TimeRequest variant:', request)
	}
}
