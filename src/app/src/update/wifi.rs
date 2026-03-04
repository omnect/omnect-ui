use std::collections::HashMap;

use crux_core::{render::render, Command};

use crate::{
    auth_get, auth_post,
    events::{
        Event, WifiEvent, WifiSavedNetworksApiResponse, WifiScanResultsApiResponse,
        WifiStatusApiResponse,
    },
    model::Model,
    types::{
        WifiAvailability, WifiConnectionState, WifiConnectionStatus, WifiNetwork, WifiSavedNetwork,
        WifiScanState, WifiState,
    },
    unauth_post, wifi_psk, Effect,
};

/// Max scan poll attempts (500ms each → 30s total)
const SCAN_POLL_MAX_ATTEMPTS: u32 = 60;
/// Max connect poll attempts (1s each → 30s total)
const CONNECT_POLL_MAX_ATTEMPTS: u32 = 30;

/// Helper to get mutable reference to the Ready variant fields
macro_rules! with_ready_state {
    ($model:expr, |$iface:ident, $status:ident, $scan_state:ident, $scan_results:ident, $saved:ident, $scan_poll:ident, $connect_poll:ident| $body:block) => {
        if let WifiState::Ready {
            interface_name: $iface,
            status: $status,
            scan_state: $scan_state,
            scan_results: $scan_results,
            saved_networks: $saved,
            scan_poll_attempt: $scan_poll,
            connect_poll_attempt: $connect_poll,
        } = &mut $model.wifi_state
        {
            $body
        } else {
            log::warn!("WiFi event received but state is Unavailable");
            Command::done()
        }
    };
}

pub fn handle(event: WifiEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        WifiEvent::CheckAvailability => {
            unauth_post!(
                Wifi, WifiEvent, model,
                "/wifi/available",
                CheckAvailabilityResponse, "Check WiFi availability",
                method: get,
                expect_json: WifiAvailability
            )
        }

        WifiEvent::CheckAvailabilityResponse(result) => match result {
            Ok(availability) if availability.available => {
                let iface = availability.interface_name.unwrap_or_default();
                model.wifi_state = WifiState::Ready {
                    interface_name: iface,
                    status: WifiConnectionStatus::default(),
                    scan_state: WifiScanState::Idle,
                    scan_results: Vec::new(),
                    saved_networks: Vec::new(),
                    scan_poll_attempt: 0,
                    connect_poll_attempt: 0,
                };
                // Only fetch status and saved networks if authenticated
                if model.is_authenticated {
                    Command::all([
                        render(),
                        handle(WifiEvent::GetStatus, model),
                        handle(WifiEvent::GetSavedNetworks, model),
                    ])
                } else {
                    render()
                }
            }
            Ok(_) => {
                // WiFi not available
                model.wifi_state = WifiState::Unavailable;
                render()
            }
            Err(e) => {
                log::error!("WiFi availability check failed: {e}");
                model.wifi_state = WifiState::Unavailable;
                render()
            }
        },

        WifiEvent::Scan => {
            with_ready_state!(model, |_iface,
                                      _status,
                                      scan_state,
                                      _scan_results,
                                      _saved,
                                      scan_poll,
                                      _connect_poll| {
                *scan_state = WifiScanState::Scanning;
                *scan_poll = 0;
                auth_post!(
                    Wifi,
                    WifiEvent,
                    model,
                    "/wifi/scan",
                    ScanResponse,
                    "WiFi scan"
                )
            })
        }

        WifiEvent::ScanResponse(result) => {
            if let Err(e) = result {
                with_ready_state!(
                    model,
                    |_iface,
                     _status,
                     scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        *scan_state = WifiScanState::Error(e);
                        render()
                    }
                )
            } else {
                // Scan started; Shell will drive polling via ScanPollTick
                render()
            }
        }

        WifiEvent::ScanPollTick => {
            with_ready_state!(model, |_iface,
                                      _status,
                                      scan_state,
                                      _scan_results,
                                      _saved,
                                      scan_poll,
                                      _connect_poll| {
                if !matches!(scan_state, WifiScanState::Scanning) {
                    return Command::done();
                }
                if *scan_poll >= SCAN_POLL_MAX_ATTEMPTS {
                    *scan_state = WifiScanState::Error("Scan timed out".to_string());
                    return render();
                }
                *scan_poll += 1;
                auth_get!(
                    Wifi, WifiEvent, model,
                    "/wifi/scan/results",
                    ScanResultsResponse, "WiFi scan results",
                    expect_json: WifiScanResultsApiResponse
                )
            })
        }

        WifiEvent::ScanResultsResponse(result) => match result {
            Ok(response) => {
                with_ready_state!(
                    model,
                    |_iface,
                     _status,
                     scan_state,
                     scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        if response.state == "finished" {
                            // Deduplicate by SSID, keeping strongest signal per SSID
                            let mut best: HashMap<String, WifiNetwork> = HashMap::new();
                            for net in response.networks {
                                let entry =
                                    best.entry(net.ssid.clone()).or_insert_with(|| WifiNetwork {
                                        ssid: net.ssid.clone(),
                                        mac: net.mac.clone(),
                                        channel: net.ch,
                                        rssi: net.rssi,
                                    });
                                if net.rssi > entry.rssi {
                                    *entry = WifiNetwork {
                                        ssid: net.ssid,
                                        mac: net.mac,
                                        channel: net.ch,
                                        rssi: net.rssi,
                                    };
                                }
                            }
                            // Sort by RSSI descending (strongest first)
                            let mut networks: Vec<WifiNetwork> = best.into_values().collect();
                            networks.sort_by(|a, b| b.rssi.cmp(&a.rssi));

                            *scan_results = networks;
                            *scan_state = WifiScanState::Finished;
                        }
                        // If state is still "scanning", keep polling (Shell timer continues)
                        render()
                    }
                )
            }
            Err(e) => {
                with_ready_state!(
                    model,
                    |_iface,
                     _status,
                     scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        *scan_state = WifiScanState::Error(e);
                        render()
                    }
                )
            }
        },

        WifiEvent::Connect { ssid, password } => {
            // Input validation
            if ssid.trim().is_empty() {
                return with_ready_state!(
                    model,
                    |_iface,
                     _status,
                     _scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        _status.state =
                            WifiConnectionState::Failed("SSID cannot be empty".to_string());
                        render()
                    }
                );
            }
            if password.is_empty() {
                return with_ready_state!(
                    model,
                    |_iface,
                     _status,
                     _scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        _status.state =
                            WifiConnectionState::Failed("Password cannot be empty".to_string());
                        render()
                    }
                );
            }

            // Compute WPA PSK
            let psk = wifi_psk::compute_wpa_psk(&password, &ssid);

            with_ready_state!(model, |_iface,
                                      status,
                                      _scan_state,
                                      _scan_results,
                                      _saved,
                                      _scan_poll,
                                      connect_poll| {
                status.state = WifiConnectionState::Connecting;
                *connect_poll = 0;

                #[derive(serde::Serialize)]
                struct ConnectBody {
                    ssid: String,
                    psk: String,
                }
                let body = ConnectBody { ssid, psk };
                auth_post!(
                    Wifi, WifiEvent, model,
                    "/wifi/connect",
                    ConnectResponse, "WiFi connect",
                    body_json: &body
                )
            })
        }

        WifiEvent::ConnectResponse(result) => {
            if let Err(e) = result {
                with_ready_state!(
                    model,
                    |_iface,
                     status,
                     _scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        status.state = WifiConnectionState::Failed(e);
                        render()
                    }
                )
            } else {
                // Connect request accepted; Shell will drive polling via ConnectPollTick
                model.stop_loading();
                render()
            }
        }

        WifiEvent::ConnectPollTick => {
            with_ready_state!(model, |_iface,
                                      status,
                                      _scan_state,
                                      _scan_results,
                                      _saved,
                                      _scan_poll,
                                      connect_poll| {
                if !matches!(status.state, WifiConnectionState::Connecting) {
                    return Command::done();
                }
                if *connect_poll >= CONNECT_POLL_MAX_ATTEMPTS {
                    status.state = WifiConnectionState::Failed("Connection timed out".to_string());
                    return render();
                }
                *connect_poll += 1;
                auth_get!(
                    Wifi, WifiEvent, model,
                    "/wifi/status",
                    StatusResponse, "WiFi connect status",
                    expect_json: WifiStatusApiResponse
                )
            })
        }

        WifiEvent::StatusResponse(result) => match result {
            Ok(response) => {
                with_ready_state!(
                    model,
                    |_iface,
                     status,
                     _scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        let was_connecting =
                            matches!(status.state, WifiConnectionState::Connecting);

                        status.ssid = response.ssid;
                        status.ip_address = response.ip_address;

                        match response.state.as_str() {
                            "connected" => {
                                status.state = WifiConnectionState::Connected;
                                if was_connecting {
                                    // Auto-refresh saved networks after successful connect
                                    return Command::all([
                                        render(),
                                        handle(WifiEvent::GetSavedNetworks, model),
                                    ]);
                                }
                            }
                            "failed" if was_connecting => {
                                status.state =
                                    WifiConnectionState::Failed("Connection failed".to_string());
                            }
                            "idle" if !was_connecting => {
                                status.state = WifiConnectionState::Idle;
                            }
                            _ => {
                                // "connecting" or other states — keep polling
                            }
                        }
                        render()
                    }
                )
            }
            Err(e) => {
                log::error!("WiFi status poll failed: {e}");
                render()
            }
        },

        WifiEvent::Disconnect => {
            auth_post!(
                Wifi,
                WifiEvent,
                model,
                "/wifi/disconnect",
                DisconnectResponse,
                "WiFi disconnect"
            )
        }

        WifiEvent::DisconnectResponse(result) => {
            model.stop_loading();
            if let Err(e) = result {
                with_ready_state!(
                    model,
                    |_iface,
                     status,
                     _scan_state,
                     _scan_results,
                     _saved,
                     _scan_poll,
                     _connect_poll| {
                        status.state = WifiConnectionState::Failed(e);
                        render()
                    }
                )
            } else {
                // Refresh status after disconnect
                handle(WifiEvent::GetStatus, model)
            }
        }

        WifiEvent::GetStatus => {
            auth_get!(
                Wifi, WifiEvent, model,
                "/wifi/status",
                StatusResponse, "WiFi status",
                expect_json: WifiStatusApiResponse
            )
        }

        WifiEvent::GetSavedNetworks => {
            auth_get!(
                Wifi, WifiEvent, model,
                "/wifi/networks",
                SavedNetworksResponse, "WiFi saved networks",
                expect_json: WifiSavedNetworksApiResponse
            )
        }

        WifiEvent::SavedNetworksResponse(result) => match result {
            Ok(response) => {
                with_ready_state!(
                    model,
                    |_iface,
                     _status,
                     _scan_state,
                     _scan_results,
                     saved,
                     _scan_poll,
                     _connect_poll| {
                        *saved = response
                            .networks
                            .into_iter()
                            .map(|n| WifiSavedNetwork {
                                ssid: n.ssid,
                                flags: n.flags,
                            })
                            .collect();
                        render()
                    }
                )
            }
            Err(e) => {
                log::error!("Failed to load saved networks: {e}");
                render()
            }
        },

        WifiEvent::ForgetNetwork { ssid } => {
            #[derive(serde::Serialize)]
            struct ForgetBody {
                ssid: String,
            }
            let body = ForgetBody { ssid };
            auth_post!(
                Wifi, WifiEvent, model,
                "/wifi/networks/forget",
                ForgetNetworkResponse, "WiFi forget network",
                body_json: &body
            )
        }

        WifiEvent::ForgetNetworkResponse(result) => {
            model.stop_loading();
            if let Err(e) = result {
                log::error!("Failed to forget network: {e}");
                render()
            } else {
                // Auto-refresh saved networks and status after forget
                Command::all([
                    render(),
                    handle(WifiEvent::GetSavedNetworks, model),
                    handle(WifiEvent::GetStatus, model),
                ])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WifiAvailability;

    fn model_with_ready_state() -> Model {
        Model {
            auth_token: Some("test-token".to_string()),
            is_authenticated: true,
            wifi_state: WifiState::Ready {
                interface_name: "wlan0".to_string(),
                status: WifiConnectionStatus::default(),
                scan_state: WifiScanState::Idle,
                scan_results: Vec::new(),
                saved_networks: Vec::new(),
                scan_poll_attempt: 0,
                connect_poll_attempt: 0,
            },
            ..Default::default()
        }
    }

    mod check_availability {
        use super::*;

        #[test]
        fn available_response_transitions_to_ready() {
            let mut model = Model::default();
            let result = Ok(WifiAvailability {
                available: true,
                interface_name: Some("wlan0".to_string()),
            });
            let _ = handle(WifiEvent::CheckAvailabilityResponse(result), &mut model);

            match &model.wifi_state {
                WifiState::Ready { interface_name, .. } => {
                    assert_eq!(interface_name, "wlan0");
                }
                _ => panic!("Expected Ready state"),
            }
        }

        #[test]
        fn unavailable_response_stays_unavailable() {
            let mut model = Model::default();
            let result = Ok(WifiAvailability {
                available: false,
                interface_name: None,
            });
            let _ = handle(WifiEvent::CheckAvailabilityResponse(result), &mut model);
            assert_eq!(model.wifi_state, WifiState::Unavailable);
        }

        #[test]
        fn error_response_stays_unavailable() {
            let mut model = Model::default();
            let result = Err("network error".to_string());
            let _ = handle(WifiEvent::CheckAvailabilityResponse(result), &mut model);
            assert_eq!(model.wifi_state, WifiState::Unavailable);
        }
    }

    mod scan {
        use super::*;
        use crate::events::WifiNetworkApiResponse;

        #[test]
        fn scan_sets_scanning_state() {
            let mut model = model_with_ready_state();
            let _ = handle(WifiEvent::Scan, &mut model);

            if let WifiState::Ready {
                scan_state,
                scan_poll_attempt,
                ..
            } = &model.wifi_state
            {
                assert_eq!(*scan_state, WifiScanState::Scanning);
                assert_eq!(*scan_poll_attempt, 0);
            } else {
                panic!("Expected Ready state");
            }
        }

        #[test]
        fn scan_error_sets_error_state() {
            let mut model = model_with_ready_state();
            let _ = handle(
                WifiEvent::ScanResponse(Err("scan failed".to_string())),
                &mut model,
            );

            if let WifiState::Ready { scan_state, .. } = &model.wifi_state {
                assert_eq!(*scan_state, WifiScanState::Error("scan failed".to_string()));
            }
        }

        #[test]
        fn scan_poll_increments_attempt() {
            let mut model = model_with_ready_state();
            // Set scanning state
            if let WifiState::Ready { scan_state, .. } = &mut model.wifi_state {
                *scan_state = WifiScanState::Scanning;
            }
            let _ = handle(WifiEvent::ScanPollTick, &mut model);

            if let WifiState::Ready {
                scan_poll_attempt, ..
            } = &model.wifi_state
            {
                assert_eq!(*scan_poll_attempt, 1);
            }
        }

        #[test]
        fn scan_poll_timeout() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready {
                scan_state,
                scan_poll_attempt,
                ..
            } = &mut model.wifi_state
            {
                *scan_state = WifiScanState::Scanning;
                *scan_poll_attempt = SCAN_POLL_MAX_ATTEMPTS;
            }
            let _ = handle(WifiEvent::ScanPollTick, &mut model);

            if let WifiState::Ready { scan_state, .. } = &model.wifi_state {
                assert!(
                    matches!(scan_state, WifiScanState::Error(msg) if msg.contains("timed out"))
                );
            }
        }

        #[test]
        fn scan_results_deduplicate_by_ssid() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready { scan_state, .. } = &mut model.wifi_state {
                *scan_state = WifiScanState::Scanning;
            }
            let response = WifiScanResultsApiResponse {
                status: "ok".to_string(),
                state: "finished".to_string(),
                networks: vec![
                    WifiNetworkApiResponse {
                        ssid: "MyNet".to_string(),
                        mac: "aa:bb:cc:dd:ee:ff".to_string(),
                        ch: 6,
                        rssi: -70,
                    },
                    WifiNetworkApiResponse {
                        ssid: "MyNet".to_string(),
                        mac: "11:22:33:44:55:66".to_string(),
                        ch: 11,
                        rssi: -50,
                    },
                    WifiNetworkApiResponse {
                        ssid: "Other".to_string(),
                        mac: "ff:ff:ff:ff:ff:ff".to_string(),
                        ch: 1,
                        rssi: -80,
                    },
                ],
            };
            let _ = handle(WifiEvent::ScanResultsResponse(Ok(response)), &mut model);

            if let WifiState::Ready {
                scan_state,
                scan_results,
                ..
            } = &model.wifi_state
            {
                assert_eq!(*scan_state, WifiScanState::Finished);
                assert_eq!(scan_results.len(), 2);
                // Strongest first
                assert_eq!(scan_results[0].ssid, "MyNet");
                assert_eq!(scan_results[0].rssi, -50);
                assert_eq!(scan_results[1].ssid, "Other");
            }
        }

        #[test]
        fn scan_results_while_still_scanning_keeps_state() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready { scan_state, .. } = &mut model.wifi_state {
                *scan_state = WifiScanState::Scanning;
            }
            let response = WifiScanResultsApiResponse {
                status: "ok".to_string(),
                state: "scanning".to_string(),
                networks: vec![],
            };
            let _ = handle(WifiEvent::ScanResultsResponse(Ok(response)), &mut model);

            if let WifiState::Ready { scan_state, .. } = &model.wifi_state {
                assert_eq!(*scan_state, WifiScanState::Scanning);
            }
        }
    }

    mod connect {
        use super::*;

        #[test]
        fn empty_ssid_fails_validation() {
            let mut model = model_with_ready_state();
            let _ = handle(
                WifiEvent::Connect {
                    ssid: "  ".to_string(),
                    password: "secret".to_string(),
                },
                &mut model,
            );

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert!(matches!(
                    &status.state,
                    WifiConnectionState::Failed(msg) if msg.contains("SSID")
                ));
            }
        }

        #[test]
        fn empty_password_fails_validation() {
            let mut model = model_with_ready_state();
            let _ = handle(
                WifiEvent::Connect {
                    ssid: "MyNet".to_string(),
                    password: "".to_string(),
                },
                &mut model,
            );

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert!(matches!(
                    &status.state,
                    WifiConnectionState::Failed(msg) if msg.contains("Password")
                ));
            }
        }

        #[test]
        fn valid_connect_sets_connecting_state() {
            let mut model = model_with_ready_state();
            let _ = handle(
                WifiEvent::Connect {
                    ssid: "MyNet".to_string(),
                    password: "password123".to_string(),
                },
                &mut model,
            );

            if let WifiState::Ready {
                status,
                connect_poll_attempt,
                ..
            } = &model.wifi_state
            {
                assert_eq!(status.state, WifiConnectionState::Connecting);
                assert_eq!(*connect_poll_attempt, 0);
            }
        }

        #[test]
        fn connect_error_sets_failed() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready { status, .. } = &mut model.wifi_state {
                status.state = WifiConnectionState::Connecting;
            }
            let _ = handle(
                WifiEvent::ConnectResponse(Err("auth failed".to_string())),
                &mut model,
            );

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert!(matches!(
                    &status.state,
                    WifiConnectionState::Failed(msg) if msg == "auth failed"
                ));
            }
        }

        #[test]
        fn connect_poll_timeout() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready {
                status,
                connect_poll_attempt,
                ..
            } = &mut model.wifi_state
            {
                status.state = WifiConnectionState::Connecting;
                *connect_poll_attempt = CONNECT_POLL_MAX_ATTEMPTS;
            }
            let _ = handle(WifiEvent::ConnectPollTick, &mut model);

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert!(matches!(
                    &status.state,
                    WifiConnectionState::Failed(msg) if msg.contains("timed out")
                ));
            }
        }

        #[test]
        fn status_connected_while_connecting_transitions() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready { status, .. } = &mut model.wifi_state {
                status.state = WifiConnectionState::Connecting;
            }
            let response = WifiStatusApiResponse {
                status: "ok".to_string(),
                state: "connected".to_string(),
                ssid: Some("MyNet".to_string()),
                ip_address: Some("192.168.1.100".to_string()),
            };
            let _ = handle(WifiEvent::StatusResponse(Ok(response)), &mut model);

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert_eq!(status.state, WifiConnectionState::Connected);
                assert_eq!(status.ssid.as_deref(), Some("MyNet"));
                assert_eq!(status.ip_address.as_deref(), Some("192.168.1.100"));
            }
        }

        #[test]
        fn status_failed_while_connecting_transitions() {
            let mut model = model_with_ready_state();
            if let WifiState::Ready { status, .. } = &mut model.wifi_state {
                status.state = WifiConnectionState::Connecting;
            }
            let response = WifiStatusApiResponse {
                status: "ok".to_string(),
                state: "failed".to_string(),
                ssid: None,
                ip_address: None,
            };
            let _ = handle(WifiEvent::StatusResponse(Ok(response)), &mut model);

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert!(matches!(
                    &status.state,
                    WifiConnectionState::Failed(msg) if msg.contains("failed")
                ));
            }
        }
    }

    mod disconnect {
        use super::*;

        #[test]
        fn disconnect_error_sets_failed() {
            let mut model = model_with_ready_state();
            let _ = handle(
                WifiEvent::DisconnectResponse(Err("disconnect failed".to_string())),
                &mut model,
            );

            if let WifiState::Ready { status, .. } = &model.wifi_state {
                assert!(matches!(
                    &status.state,
                    WifiConnectionState::Failed(msg) if msg == "disconnect failed"
                ));
            }
        }
    }

    mod forget_network {
        use super::*;
        use crate::events::WifiSavedNetworkApiResponse;

        #[test]
        fn forget_error_logs_and_renders() {
            let mut model = model_with_ready_state();
            let _ = handle(
                WifiEvent::ForgetNetworkResponse(Err("not found".to_string())),
                &mut model,
            );
            // No crash, renders
        }

        #[test]
        fn saved_networks_response_updates_state() {
            let mut model = model_with_ready_state();
            let response = WifiSavedNetworksApiResponse {
                status: "ok".to_string(),
                networks: vec![
                    WifiSavedNetworkApiResponse {
                        ssid: "Home".to_string(),
                        flags: "[CURRENT]".to_string(),
                    },
                    WifiSavedNetworkApiResponse {
                        ssid: "Work".to_string(),
                        flags: "".to_string(),
                    },
                ],
            };
            let _ = handle(WifiEvent::SavedNetworksResponse(Ok(response)), &mut model);

            if let WifiState::Ready { saved_networks, .. } = &model.wifi_state {
                assert_eq!(saved_networks.len(), 2);
                assert_eq!(saved_networks[0].ssid, "Home");
                assert_eq!(saved_networks[0].flags, "[CURRENT]");
            }
        }
    }

    mod unavailable_state {
        use super::*;

        #[test]
        fn scan_on_unavailable_state_returns_done() {
            let mut model = Model::default();
            assert_eq!(model.wifi_state, WifiState::Unavailable);
            let _ = handle(WifiEvent::Scan, &mut model);
            // Should not panic, just returns done
        }
    }
}
