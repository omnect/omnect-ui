/**
 * Timer management for device operations and network changes
 *
 * This module handles:
 * - Reconnection timeout and countdown after reboot/factory reset
 * - New IP timeout and countdown after network config changes
 * - Navigation when new IP becomes reachable
 *
 * Polling (ReconnectionCheckTick, NewIpCheckTick, ScanPollTick, ConnectPollTick)
 * is now scheduled by Core via crux_time and no longer managed here.
 */

import { watch } from 'vue'
import { viewModel } from './state'
import type { Event } from '../../../../shared_types/generated/typescript/types/shared_types'
import {
	EventVariantDevice,
	DeviceEventVariantReconnectionTimeout,
	DeviceEventVariantNewIpCheckTimeout,
} from '../../../../shared_types/generated/typescript/types/shared_types'

// Timer callback type - will be set by index.ts to avoid circular dependency
let sendEventCallback: ((event: Event) => Promise<void>) | null = null

/**
 * Set the event sender callback (called from index.ts after initialization)
 */
export function setEventSender(callback: (event: Event) => Promise<void>): void {
	sendEventCallback = callback
}

// ============================================================================
// Timer Constants
// ============================================================================

// Optional test overrides for reconnection timeouts (production values come from Core)
const REBOOT_TIMEOUT_OVERRIDE_MS = import.meta.env.VITE_REBOOT_TIMEOUT_MS ? Number(import.meta.env.VITE_REBOOT_TIMEOUT_MS) : null
const FACTORY_RESET_TIMEOUT_OVERRIDE_MS = import.meta.env.VITE_FACTORY_RESET_TIMEOUT_MS ? Number(import.meta.env.VITE_FACTORY_RESET_TIMEOUT_MS) : null
const FIRMWARE_UPDATE_TIMEOUT_OVERRIDE_MS = import.meta.env.VITE_FIRMWARE_UPDATE_TIMEOUT_MS ? Number(import.meta.env.VITE_FIRMWARE_UPDATE_TIMEOUT_MS) : null

// ============================================================================
// Timer IDs
// ============================================================================

let reconnectionTimeoutId: ReturnType<typeof setTimeout> | null = null
let reconnectionCountdownIntervalId: ReturnType<typeof setInterval> | null = null
let reconnectionCountdownDeadline: number | null = null
let newIpTimeoutId: ReturnType<typeof setTimeout> | null = null
let newIpCountdownIntervalId: ReturnType<typeof setInterval> | null = null

// Countdown deadline for network changes (Unix timestamp in milliseconds)
let countdownDeadline: number | null = null

// ============================================================================
// Reconnection Timeout + Countdown
// ============================================================================

/**
 * Start reconnection timeout and countdown for reboot/factory reset/update.
 * Polling (ReconnectionCheckTick) is scheduled by Core via crux_time.
 */
export function startReconnectionPolling(): void {
	// Read timeout from Core's overlay spinner countdown BEFORE clearing (stop clears countdownSeconds)
	const coreCountdownSeconds = viewModel.overlaySpinner.countdownSeconds

	stopReconnectionPolling() // Clear any existing timers
	if (!coreCountdownSeconds || coreCountdownSeconds <= 0) {
		console.warn('[useCore] startReconnectionPolling: no countdown from Core, skipping timeout')
		return
	}

	// Allow test env override for shorter timeouts
	const isFactoryReset = viewModel.deviceOperationState.type === 'factoryResetting'
	const isFirmwareUpdate = viewModel.deviceOperationState.type === 'updating'
	const overrideMs = isFactoryReset ? FACTORY_RESET_TIMEOUT_OVERRIDE_MS
		: isFirmwareUpdate ? FIRMWARE_UPDATE_TIMEOUT_OVERRIDE_MS
		: REBOOT_TIMEOUT_OVERRIDE_MS
	const timeoutMs = overrideMs ?? coreCountdownSeconds * 1000
	const countdownSeconds = Math.ceil(timeoutMs / 1000)
	// Update the displayed countdown to match effective timeout
	viewModel.overlaySpinner.countdownSeconds = countdownSeconds
	console.log(`[useCore] Starting reconnection timeout (${countdownSeconds}s)`)

	// Set countdown deadline
	reconnectionCountdownDeadline = Date.now() + timeoutMs

	// Start countdown interval (1 second for UI countdown)
	reconnectionCountdownIntervalId = setInterval(() => {
		if (reconnectionCountdownDeadline !== null) {
			const remainingMs = Math.max(0, reconnectionCountdownDeadline - Date.now())
			const remainingSeconds = Math.ceil(remainingMs / 1000)
			viewModel.overlaySpinner.countdownSeconds = remainingSeconds
		}
	}, 1000)

	// Set timeout
	reconnectionTimeoutId = setTimeout(() => {
		console.log('[useCore] Reconnection timeout reached')
		if (sendEventCallback) {
			sendEventCallback(new EventVariantDevice(new DeviceEventVariantReconnectionTimeout()))
		}
		stopReconnectionPolling()
	}, timeoutMs)
}

/**
 * Stop reconnection timeout and countdown
 */
export function stopReconnectionPolling(): void {
	if (reconnectionTimeoutId !== null) {
		clearTimeout(reconnectionTimeoutId)
		reconnectionTimeoutId = null
	}
	if (reconnectionCountdownIntervalId !== null) {
		clearInterval(reconnectionCountdownIntervalId)
		reconnectionCountdownIntervalId = null
	}
	reconnectionCountdownDeadline = null
	viewModel.overlaySpinner.countdownSeconds = null
}

// ============================================================================
// New IP Timeout + Countdown
// ============================================================================

/**
 * Start new IP timeout and countdown after network config change.
 * Polling (NewIpCheckTick) is scheduled by Core via crux_time.
 */
export function startNewIpPolling(): void {
	stopNewIpPolling() // Clear any existing timers

	console.log('[useCore] Starting new IP timeout/countdown')

	// Clear messages when starting so that arriving at new IP/re-login
	// doesn't have stale success/error state
	viewModel.successMessage = null
	viewModel.errorMessage = null

	const state = viewModel.networkChangeState
	if (!state || (state.type !== 'waitingForNewIp' && state.type !== 'waitingForOldIp')) {
		console.warn('[useCore] startNewIpPolling called but state is not waitingForNewIp or waitingForOldIp:', state)
		return
	}

	let rollbackTimeout = 0
	if (state.type === 'waitingForNewIp') {
		rollbackTimeout = (state as any).rollbackTimeoutSeconds
	}

	const timeoutMs = rollbackTimeout * 1000

	// Set countdown deadline
	countdownDeadline = Date.now() + timeoutMs

	// Only start countdown and timeout if rollback is enabled (timeout > 0)
	if (rollbackTimeout > 0) {
		// Update countdown immediately
		if (countdownDeadline !== null) {
			const remainingMs = Math.max(0, countdownDeadline - Date.now())
			viewModel.overlaySpinner.countdownSeconds = Math.ceil(remainingMs / 1000)
		}

		// Start countdown interval (every 1 second for UI countdown)
		newIpCountdownIntervalId = setInterval(() => {
			if (countdownDeadline !== null) {
				const remainingMs = Math.max(0, countdownDeadline - Date.now())
				viewModel.overlaySpinner.countdownSeconds = Math.ceil(remainingMs / 1000)
			}
		}, 1000)

		// Set timeout
		newIpTimeoutId = setTimeout(() => {
			console.log('[useCore] New IP timeout reached')
			if (sendEventCallback) {
				sendEventCallback(new EventVariantDevice(new DeviceEventVariantNewIpCheckTimeout()))
			}
			stopNewIpPolling()
		}, timeoutMs)
	}
}

/**
 * Stop new IP timeout and countdown
 */
export function stopNewIpPolling(): void {
	if (newIpCountdownIntervalId !== null) {
		clearInterval(newIpCountdownIntervalId)
		newIpCountdownIntervalId = null
	}
	if (newIpTimeoutId !== null) {
		clearTimeout(newIpTimeoutId)
		newIpTimeoutId = null
	}
	// Clear countdown seconds in viewModel
	viewModel.overlaySpinner.countdownSeconds = null
	// Clear countdown deadline
	countdownDeadline = null
}

// ============================================================================
// State Watchers
// ============================================================================

/**
 * Initialize watchers for automatic timer management.
 * Call this once during module initialization.
 */
export function initializeTimerWatchers(): void {
	// Watch deviceOperationState for reconnection timeout/countdown
	watch(
		() => viewModel.deviceOperationState,
		(newState, oldState) => {
			const newType = newState?.type
			const oldType = oldState?.type

			// Only act on type transitions
			if (newType === oldType) return

			// Start timeout when entering rebooting, factoryResetting, or updating state
			if (newType === 'rebooting' || newType === 'factoryResetting' || newType === 'updating') {
				startReconnectionPolling()
			}
			// Stop timeout when leaving these states or entering terminal states
			else if (
				(oldType === 'rebooting' || oldType === 'factoryResetting' || oldType === 'updating' || oldType === 'waitingReconnection') &&
				(newType === 'idle' || newType === 'reconnectionSuccessful' || newType === 'reconnectionFailed')
			) {
				stopReconnectionPolling()
			}
		},
		{ deep: true }
	)

	// Watch networkChangeState for new IP timeout/countdown and redirect
	watch(
		() => viewModel.networkChangeState,
		(newState, oldState) => {
			const newType = newState?.type
			const oldType = oldState?.type

			const isPollingState = (type: string | undefined) =>
				type === 'waitingForNewIp' || type === 'waitingForOldIp'

			if (isPollingState(newType) && !isPollingState(oldType)) {
				startNewIpPolling()
			} else if (isPollingState(oldType) && !isPollingState(newType)) {
				stopNewIpPolling()
			} else if (isPollingState(newType) && isPollingState(oldType) && newType !== oldType) {
				// Switching between polling states (e.g. waitingForNewIp → waitingForOldIp):
				// restart to update timeout config
				startNewIpPolling()
			}

			// Navigate to new IP when it's reachable
			if (newState?.type === 'newIpReachable') {
				console.log(`[useCore] Redirecting to new IP: ${newState.newIp}:${newState.uiPort}`)
				viewModel.successMessage = null
				viewModel.errorMessage = null
				window.location.href = `https://${newState.newIp}:${newState.uiPort}`
			}
		},
		{ deep: true }
	)
}