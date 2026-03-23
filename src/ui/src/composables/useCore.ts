/**
 * Vue composable for integrating with Crux Core
 *
 * @see ./core/index.ts - Main composable and public API
 * @see ./core/types.ts - Type definitions and conversions
 * @see ./core/state.ts - Singleton reactive state
 * @see ./core/effects.ts - Effect processing
 * @see ./core/http.ts - HTTP capability
 * @see ./core/websocket.ts - WebSocket capability
 * @see ./core/time.ts - Time/timer capability (crux_time)
 * @see ./core/sync.ts - ViewModel synchronization and navigation side-effects
 */

// Re-export everything from core/index.ts
export { useCore } from './core'

// Re-export types
export type {
	ViewModel,
	DeviceOperationStateType,
	NetworkChangeStateType,
	NetworkFormStateType,
	NetworkFormDataType,
	OverlaySpinnerStateType,
	FactoryResetStatusString,
	WifiStateType,
	WifiNetworkType,
	WifiSavedNetworkType,
	WifiConnectionStatusType,
	WifiScanStateType,
	WifiConnectionStateType,
	SystemInfo,
	NetworkStatus,
	OnlineStatus,
	FactoryReset,
	UpdateValidationStatus,
	Timeouts,
	HealthcheckInfo,
	Event,
	Effect,
	CoreViewModel,
	UpdateManifest,
	NetworkFormData,
	DeviceNetwork,
} from './core'

// Re-export NetworkConfigRequest class
export { NetworkConfigRequest } from './core'
