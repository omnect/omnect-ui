/**
 * Timer management for device operations and network changes
 *
 * This module handles:
 * - Reconnection polling after reboot/factory reset
 * - New IP polling after network config changes
 * - Automatic timeout handling
 */

import { watch } from 'vue'
import { viewModel, isInitialized, wasmModule } from './state'
import type { Event } from '../../../../shared_types/generated/typescript/types/shared_types'
import {
	EventVariantDevice,
	DeviceEventVariantReconnectionCheckTick,
	DeviceEventVariantReconnectionTimeout,
	DeviceEventVariantNewIpCheckTick,
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

const RECONNECTION_POLL_INTERVAL_MS = 5000 // 5 seconds
const REBOOT_TIMEOUT_MS = 300000 // 5 minutes
const FACTORY_RESET_TIMEOUT_MS = 600000 // 10 minutes
const NEW_IP_POLL_INTERVAL_MS = 5000 // 5 seconds
const NEW_IP_TIMEOUT_MS = 90000 // 90 seconds

// ============================================================================
// Timer IDs
// ============================================================================

let reconnectionIntervalId: ReturnType<typeof setInterval> | null = null
let reconnectionTimeoutId: ReturnType<typeof setTimeout> | null = null
let newIpIntervalId: ReturnType<typeof setInterval> | null = null
let newIpTimeoutId: ReturnType<typeof setTimeout> | null = null

// ============================================================================
// Reconnection Polling
// ============================================================================

/**
 * Start reconnection polling for reboot/factory reset
 * Sends ReconnectionCheckTick every 5 seconds and sets a timeout
 */
export function startReconnectionPolling(isFactoryReset: boolean): void {
	stopReconnectionPolling() // Clear any existing timers

	console.log(`[useCore] Starting reconnection polling (${isFactoryReset ? 'factory reset' : 'reboot'})`)

	// Start polling interval
	reconnectionIntervalId = setInterval(() => {
		if (isInitialized.value && wasmModule && sendEventCallback) {
			sendEventCallback(new EventVariantDevice(new DeviceEventVariantReconnectionCheckTick()))
		}
	}, RECONNECTION_POLL_INTERVAL_MS)

	// Set timeout (factory reset uses longer timeout)
	const timeoutMs = isFactoryReset ? FACTORY_RESET_TIMEOUT_MS : REBOOT_TIMEOUT_MS
	reconnectionTimeoutId = setTimeout(() => {
		console.log('[useCore] Reconnection timeout reached')
		if (isInitialized.value && wasmModule && sendEventCallback) {
			sendEventCallback(new EventVariantDevice(new DeviceEventVariantReconnectionTimeout()))
		}
		stopReconnectionPolling()
	}, timeoutMs)
}

/**
 * Stop reconnection polling
 */
export function stopReconnectionPolling(): void {
	if (reconnectionIntervalId !== null) {
		clearInterval(reconnectionIntervalId)
		reconnectionIntervalId = null
	}
	if (reconnectionTimeoutId !== null) {
		clearTimeout(reconnectionTimeoutId)
		reconnectionTimeoutId = null
	}
}

// ============================================================================
// New IP Polling
// ============================================================================

/**
 * Start new IP polling after network config change
 * Sends NewIpCheckTick every 5 seconds and sets a 90-second timeout
 */
export function startNewIpPolling(): void {
	stopNewIpPolling() // Clear any existing timers

	console.log('[useCore] Starting new IP polling')

	// Start polling interval
	newIpIntervalId = setInterval(() => {
		if (isInitialized.value && wasmModule && sendEventCallback) {
			sendEventCallback(new EventVariantDevice(new DeviceEventVariantNewIpCheckTick()))
		}
	}, NEW_IP_POLL_INTERVAL_MS)

	// Set timeout
	newIpTimeoutId = setTimeout(() => {
		console.log('[useCore] New IP polling timeout reached')
		if (isInitialized.value && wasmModule && sendEventCallback) {
			sendEventCallback(new EventVariantDevice(new DeviceEventVariantNewIpCheckTimeout()))
		}
		stopNewIpPolling()
	}, NEW_IP_TIMEOUT_MS)
}

/**
 * Stop new IP polling
 */
export function stopNewIpPolling(): void {
	if (newIpIntervalId !== null) {
		clearInterval(newIpIntervalId)
		newIpIntervalId = null
	}
	if (newIpTimeoutId !== null) {
		clearTimeout(newIpTimeoutId)
		newIpTimeoutId = null
	}
}

// ============================================================================
// State Watchers
// ============================================================================

/**
 * Initialize watchers for automatic timer management
 * Call this once during module initialization
 */
export function initializeTimerWatchers(): void {
	// Watch device_operation_state for reconnection polling
	watch(
		() => viewModel.device_operation_state,
		(newState, oldState) => {
			const newType = newState?.type
			const oldType = oldState?.type

			// Start polling when entering rebooting, factory_resetting, or updating state
			if (newType === 'rebooting' || newType === 'factory_resetting' || newType === 'updating') {
				startReconnectionPolling(newType === 'factory_resetting')
			}
			// Stop polling when leaving these states or entering terminal states
			else if (
				(oldType === 'rebooting' || oldType === 'factory_resetting' || oldType === 'updating' || oldType === 'waiting_reconnection') &&
				(newType === 'idle' || newType === 'reconnection_successful' || newType === 'reconnection_failed')
			) {
				stopReconnectionPolling()
			}
		},
		{ deep: true }
	)

	// Watch network_change_state for new IP polling and redirect
	watch(
		() => viewModel.network_change_state,
		(newState, oldState) => {
			const newType = newState?.type
			const oldType = oldState?.type

			// Start polling when entering waiting_for_new_ip state
			if (newType === 'waiting_for_new_ip') {
				startNewIpPolling()
			}
			// Stop polling when leaving waiting_for_new_ip state
			else if (oldType === 'waiting_for_new_ip') {
				stopNewIpPolling()
			}

			// Navigate to new IP when it's reachable
			if (newType === 'new_ip_reachable' && 'new_ip' in newState) {
				const newIp = newState.new_ip
				console.log(`[useCore] Redirecting to new IP: ${newIp}`)
				const port = window.location.port
				const protocol = window.location.protocol
				window.location.replace(`${protocol}//${newIp}${port ? `:${port}` : ''}`)
			}
		},
		{ deep: true }
	)
}
