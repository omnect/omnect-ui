use anyhow::Result;
use crux_core::typegen::TypeGen;
use omnect_ui_core::{
    events::{AuthEvent, DeviceEvent, UiEvent, WebSocketEvent, WifiEvent},
    types::{
        DeviceOperationState, FactoryResetStatus, NetworkChangeState, NetworkConfigRequest,
        NetworkFormData, NetworkFormState, TimeoutSettings, UploadState, WifiConnectionState,
        WifiConnectionStatus, WifiNetwork, WifiSavedNetwork, WifiScanState, WifiState,
    },
    App,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../app");

    let mut typegen = TypeGen::new();

    typegen.register_app::<App>()?;

    // Explicitly register domain event enums to ensure all variants are traced
    typegen.register_type::<AuthEvent>()?;
    typegen.register_type::<DeviceEvent>()?;
    typegen.register_type::<WebSocketEvent>()?;
    typegen.register_type::<UiEvent>()?;
    typegen.register_type::<WifiEvent>()?;

    // Explicitly register other enums/structs to ensure all variants are traced
    typegen.register_type::<FactoryResetStatus>()?;
    typegen.register_type::<DeviceOperationState>()?;
    typegen.register_type::<NetworkChangeState>()?;
    typegen.register_type::<NetworkFormState>()?;
    typegen.register_type::<UploadState>()?;
    typegen.register_type::<NetworkConfigRequest>()?;
    typegen.register_type::<NetworkFormData>()?;
    typegen.register_type::<TimeoutSettings>()?;
    typegen.register_type::<omnect_ui_core::types::WebSocketChannel>()?;

    // Register WiFi types
    typegen.register_type::<WifiState>()?;
    typegen.register_type::<WifiScanState>()?;
    typegen.register_type::<WifiConnectionState>()?;
    typegen.register_type::<WifiConnectionStatus>()?;
    typegen.register_type::<WifiNetwork>()?;
    typegen.register_type::<WifiSavedNetwork>()?;

    // Register ODS types
    typegen.register_type::<omnect_ui_core::types::OdsOnlineStatus>()?;
    typegen.register_type::<omnect_ui_core::types::OdsSystemInfo>()?;
    typegen.register_type::<omnect_ui_core::types::OdsTimeouts>()?;
    typegen.register_type::<omnect_ui_core::types::OdsNetworkStatus>()?;
    typegen.register_type::<omnect_ui_core::types::OdsFactoryReset>()?;
    typegen.register_type::<omnect_ui_core::types::OdsUpdateValidationStatus>()?;

    let output_root = PathBuf::from("./generated");

    typegen.typescript("shared_types", output_root.join("typescript"))?;

    Ok(())
}
