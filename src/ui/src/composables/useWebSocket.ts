import { type Ref, ref } from "vue"
import { webSocketChannelToString, type WebSocketChannel } from "./core/types"
import { useEventHook } from "./useEventHook"

// Global state for the token ref, passed from useCore
let globalAuthTokenRef: Ref<string | null> | undefined

const ws: Ref<WebSocket | undefined> = ref(undefined)
const connectedEvent = useEventHook()
const isConnected = ref(false)

const subscriptions = new Map<string, Array<(data: any) => void>>()

export function useWebSocket() {
	const setAuthToken = (tokenRef: Ref<string | null>) => {
		globalAuthTokenRef = tokenRef
	}

	const initializeWebSocket = () => {
		if (ws.value && ws.value.readyState !== WebSocket.CLOSED) {
			return
		}

		if (!globalAuthTokenRef) {
			console.error("WebSocket initialization error: authTokenRef not set. Call setAuthToken first.")
			return
		}

		const protocol = window.location.protocol === "https:" ? "wss:" : "ws:"
		const ws_url = `${protocol}//${window.location.host}/ws`

		ws.value = new WebSocket(ws_url)

		ws.value.onopen = () => {
			isConnected.value = true
			connectedEvent.trigger()
			console.debug(`[WebSocket] connected to ${ws_url}`)
		}

		ws.value.onclose = (event) => {
			isConnected.value = false
			console.debug(`[WebSocket] disconnected: ${event.code}, ${event.reason}`)
			ws.value = undefined
		}

		ws.value.onerror = (error) => {
			console.error("[WebSocket] error:", error)
		}

		ws.value.onmessage = (event) => {
			try {
				const payload = JSON.parse(event.data)
				if (payload && payload.channel && payload.data) {
					const callbacks = subscriptions.get(payload.channel)
					if (callbacks) {
						for (const cb of callbacks) {
							cb(payload.data)
						}
					}
				}
			} catch (e) {
				console.error("[WebSocket] failed to parse message", e)
			}
		}
	}

	const disconnect = () => {
		if (ws.value) {
			ws.value.close()
			ws.value = undefined
			isConnected.value = false
		}
	}

	const subscribe = async <T>(callback: (data: T) => void, channel: WebSocketChannel | string) => {
		const channelName = typeof channel === 'string' ? channel : webSocketChannelToString(channel)
		const subs = subscriptions.get(channelName) || []
		subs.push(callback)
		subscriptions.set(channelName, subs)
	}

	const history = async <T>(callback: (data: T) => void, channel: WebSocketChannel | string) => {
		const channelName = typeof channel === 'string' ? channel : webSocketChannelToString(channel)
		// History is not natively supported without Centrifugo.
		// For our use cases (mostly status updates), the first fresh update via
		// the active websocket connection will sync the state soon enough.
		console.debug(`[WebSocket] History requested for ${channelName}, but not implemented in native WS.`)
	}

	const unsubscribe = (channel: WebSocketChannel | string) => {
		const channelName = typeof channel === 'string' ? channel : webSocketChannelToString(channel)
		subscriptions.delete(channelName)
	}

	const unsubscribeAll = () => {
		subscriptions.clear()
	}

	return { subscribe, unsubscribe, unsubscribeAll, initializeWebSocket, history, disconnect, onConnected: connectedEvent.on, isConnected, setAuthToken }
}