/**
 * WebSocket capability implementation for Crux Core
 *
 * This module handles WebSocket subscriptions and message parsing
 * for real-time updates from omnect-device-service (ODS).
 */

import { websocketInstance, wasmModule } from './state'
import {
	webSocketChannelToString,
	WebSocketChannel,
	WebSocketChannelVariantOnlineStatusV1,
	WebSocketChannelVariantSystemInfoV1,
	WebSocketChannelVariantTimeoutsV1,
	WebSocketChannelVariantNetworkStatusV1,
	WebSocketChannelVariantFactoryResetV1,
	WebSocketChannelVariantUpdateValidationStatusV1,
} from './types'
import {
	EventVariantWebSocket,
	WebSocketEventVariantOnlineStatusUpdated,
	WebSocketEventVariantSystemInfoUpdated,
	WebSocketEventVariantNetworkStatusUpdated,
	WebSocketEventVariantFactoryResetUpdated,
	WebSocketEventVariantUpdateValidationStatusUpdated,
	WebSocketEventVariantTimeoutsUpdated,
	WebSocketOperationVariantSubscribe,
	WebSocketOperationVariantUnsubscribe,
	WebSocketOperationVariantSubscribeAll,
	WebSocketOperationVariantUnsubscribeAll,
	WebSocketOutputVariantConnected,
	WebSocketOutputVariantDisconnected,
	WebSocketOutputVariantError,
	type Event,
} from '../../../../shared_types/generated/typescript/types/shared_types'
import { BincodeSerializer } from '../../../../shared_types/generated/typescript/bincode/mod'

// Event sender callback - set by index.ts to avoid circular dependency
let sendEventCallback: ((event: Event) => Promise<void>) | null = null

// Effects processor callback - set by effects.ts to avoid circular dependency
let processEffectsCallback: ((effectsBytes: Uint8Array) => Promise<void>) | null = null

/**
 * Set the event sender callback (called from index.ts after initialization)
 */
export function setEventSender(callback: (event: Event) => Promise<void>): void {
	sendEventCallback = callback
}

/**
 * Set the effects processor callback (called from effects.ts)
 */
export function setEffectsProcessor(callback: (effectsBytes: Uint8Array) => Promise<void>): void {
	processEffectsCallback = callback
}

/**
 * Parse WebSocket channel data from ODS JSON and send as typed event to Core
 *
 * Architecture:
 * - Receives JSON from WebSocket WebSocket (ODS data format)
 * - Sends raw JSON string to Core
 * - Core parses JSON and constructs internal types
 * - Core processes events, updates Model, and renders
 * - Shell reads updated viewModel from Core
 *
 * This event-based approach avoids request/response conflicts with streaming data.
 */
async function parseAndSendChannelEvent(channel: string, jsonData: string): Promise<void> {
	if (!sendEventCallback) {
		console.warn('[WebSocket] Event sender not initialized')
		return
	}

	try {
		switch (channel) {
			case 'OnlineStatusV1': {
				await sendEventCallback(new EventVariantWebSocket(new WebSocketEventVariantOnlineStatusUpdated(jsonData)))
				break
			}
			case 'SystemInfoV1': {
				await sendEventCallback(new EventVariantWebSocket(new WebSocketEventVariantSystemInfoUpdated(jsonData)))
				break
			}
			case 'TimeoutsV1': {
				await sendEventCallback(new EventVariantWebSocket(new WebSocketEventVariantTimeoutsUpdated(jsonData)))
				break
			}
			case 'NetworkStatusV1': {
				await sendEventCallback(new EventVariantWebSocket(new WebSocketEventVariantNetworkStatusUpdated(jsonData)))
				break
			}
			case 'FactoryResetV1': {
				await sendEventCallback(new EventVariantWebSocket(new WebSocketEventVariantFactoryResetUpdated(jsonData)))
				break
			}
			case 'UpdateValidationStatusV1': {
				await sendEventCallback(
					new EventVariantWebSocket(new WebSocketEventVariantUpdateValidationStatusUpdated(jsonData))
				)
				break
			}
			default:
				console.warn(`[WebSocket] Unknown channel: ${channel}`)
		}
	} catch (error) {
		console.error(`[WebSocket] Error parsing ${channel}:`, error)
	}
}

/**
 * Execute WebSocket operation
 *
 * Subscribes to all WebSocket channels and forwards messages as events to Core.
 * Uses the event-based architecture where WebSocket data is parsed and sent as
 * typed events (*Updated) rather than responses.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function executeWebSocketOperation(requestId: number, operation: any): Promise<void> {
	const sendResponse = async (output: any) => {
		if (!wasmModule.value) return
		const serializer = new BincodeSerializer()
		output.serialize(serializer)
		const responseBytes = serializer.getBytes()
		const newEffectsBytes = wasmModule.value.handle_response(requestId, responseBytes) as Uint8Array
		if (newEffectsBytes.length > 0 && processEffectsCallback) {
			await processEffectsCallback(newEffectsBytes)
		}
	}

	const allChannels = [
		new WebSocketChannelVariantOnlineStatusV1(),
		new WebSocketChannelVariantSystemInfoV1(),
		new WebSocketChannelVariantTimeoutsV1(),
		new WebSocketChannelVariantNetworkStatusV1(),
		new WebSocketChannelVariantFactoryResetV1(),
		new WebSocketChannelVariantUpdateValidationStatusV1(),
	]

	if (operation instanceof WebSocketOperationVariantSubscribeAll) {
		websocketInstance.initializeWebSocket()

		let subscriptionsStarted = false
		const performSubscriptions = async () => {
			if (subscriptionsStarted) return
			subscriptionsStarted = true

			try {
				for (const channel of allChannels) {
					const channelName = webSocketChannelToString(channel)
					await websocketInstance.subscribe((data: unknown) => {
						const jsonData = JSON.stringify(data)
						parseAndSendChannelEvent(channelName, jsonData)
					}, channelName)
				}
				await sendResponse(new WebSocketOutputVariantConnected())
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : String(error)
				await sendResponse(new WebSocketOutputVariantError(errorMessage))
			}
		}

		websocketInstance.onConnected(() => {
			performSubscriptions()
		})
		performSubscriptions()
	} else if (operation instanceof WebSocketOperationVariantSubscribe) {
		const channelName = webSocketChannelToString(operation.channel)
		await websocketInstance.subscribe((data: unknown) => {
			const jsonData = JSON.stringify(data)
			parseAndSendChannelEvent(channelName, jsonData)
		}, channelName)
		// Shell-only response for individual subscribe
	} else if (operation instanceof WebSocketOperationVariantUnsubscribe) {
		const channelName = webSocketChannelToString(operation.channel)
		websocketInstance.unsubscribe(channelName)
	} else if (operation instanceof WebSocketOperationVariantUnsubscribeAll) {
		websocketInstance.disconnect()
		await sendResponse(new WebSocketOutputVariantDisconnected())
	} else {
		console.error(`[WebSocket] Unsupported operation`)
		await sendResponse(new WebSocketOutputVariantError('Unsupported operation'))
	}
}