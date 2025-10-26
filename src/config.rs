use anyhow::{Context, Result, bail};
use std::{env, path::PathBuf};
use uuid::Uuid;

/// Application configuration loaded and validated at startup
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// UI server configuration
    pub ui: UiConfig,

    /// Centrifugo WebSocket server configuration
    pub centrifugo: CentrifugoConfig,

    /// Keycloak SSO configuration
    pub keycloak: KeycloakConfig,

    /// Device service client configuration
    pub device_service: DeviceServiceConfig,

    /// TLS certificate configuration
    pub certificate: CertificateConfig,

    /// IoT Edge workload API configuration (optional)
    pub iot_edge: Option<IoTEdgeConfig>,

    /// Path configuration
    pub paths: PathConfig,

    /// Tenant identifier
    pub tenant: String,
}

#[derive(Clone, Debug)]
pub struct UiConfig {
    pub port: u16,
}

#[derive(Clone, Debug)]
pub struct CentrifugoConfig {
    pub port: String,
    pub client_token: String,
    pub api_key: String,
}

#[derive(Clone, Debug)]
pub struct KeycloakConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct DeviceServiceConfig {
    pub socket_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct CertificateConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct IoTEdgeConfig {
    pub module_id: String,
    pub module_generation_id: String,
    pub api_version: String,
    pub workload_uri: String,
}

#[derive(Clone, Debug)]
pub struct PathConfig {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub host_data_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

impl AppConfig {
    /// Load and validate all configuration from environment variables
    ///
    /// This should be called once at application startup. It validates all
    /// required environment variables and returns an error if any are missing
    /// or invalid.
    pub fn load() -> Result<Self> {
        // Validate critical paths exist before proceeding
        Self::validate_filesystem()?;

        let ui = UiConfig::load()?;
        let centrifugo = CentrifugoConfig::load()?;
        let keycloak = KeycloakConfig::load()?;
        let device_service = DeviceServiceConfig::load()?;
        let certificate = CertificateConfig::load()?;
        let iot_edge = IoTEdgeConfig::load_optional();
        let paths = PathConfig::load()?;
        let tenant = env::var("TENANT").unwrap_or_else(|_| "cp".to_string());

        Ok(Self {
            ui,
            centrifugo,
            keycloak,
            device_service,
            certificate,
            iot_edge,
            paths,
            tenant,
        })
    }

    fn validate_filesystem() -> Result<()> {
        if !std::fs::exists("/data").is_ok_and(|ok| ok) {
            bail!("failed to find required data directory: /data is missing");
        }
        Ok(())
    }
}

impl UiConfig {
    fn load() -> Result<Self> {
        let port = env::var("UI_PORT")
            .context("failed to read UI_PORT environment variable")?
            .parse::<u16>()
            .context("failed to parse UI_PORT: invalid format")?;

        Ok(Self { port })
    }
}

impl CentrifugoConfig {
    fn load() -> Result<Self> {
        let port = env::var("CENTRIFUGO_HTTP_SERVER_PORT").unwrap_or_else(|_| "8000".to_string());

        // Generate unique tokens for this instance
        let client_token = Uuid::new_v4().to_string();
        let api_key = Uuid::new_v4().to_string();

        Ok(Self {
            port,
            client_token,
            api_key,
        })
    }
}

impl KeycloakConfig {
    fn load() -> Result<Self> {
        let url = env::var("KEYCLOAK_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080/realms/omnect".to_string());

        Ok(Self { url })
    }
}

impl DeviceServiceConfig {
    fn load() -> Result<Self> {
        let socket_path = env::var("SOCKET_PATH")
            .unwrap_or_else(|_| "/socket/api.sock".to_string())
            .into();

        Ok(Self { socket_path })
    }
}

impl CertificateConfig {
    fn load() -> Result<Self> {
        let cert_path = env::var("CERT_PATH")
            .unwrap_or_else(|_| "/cert/cert.pem".to_string())
            .into();

        let key_path = env::var("KEY_PATH")
            .unwrap_or_else(|_| "/cert/key.pem".to_string())
            .into();

        Ok(Self {
            cert_path,
            key_path,
        })
    }
}

impl IoTEdgeConfig {
    /// Load IoT Edge configuration if running in IoT Edge environment
    /// Returns None if not in IoT Edge environment
    fn load_optional() -> Option<Self> {
        let module_id = env::var("IOTEDGE_MODULEID").ok()?;
        let module_generation_id = env::var("IOTEDGE_MODULEGENERATIONID").ok()?;
        let api_version = env::var("IOTEDGE_APIVERSION").ok()?;
        let workload_uri = env::var("IOTEDGE_WORKLOADURI").ok()?;

        Some(Self {
            module_id,
            module_generation_id,
            api_version,
            workload_uri,
        })
    }
}

impl PathConfig {
    fn load() -> Result<Self> {
        let config_dir = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "/data/config".to_string())
            .into();

        let data_dir = PathBuf::from("/data/");
        let host_data_dir = PathBuf::from(format!("/var/lib/{}/", env!("CARGO_PKG_NAME")));
        let tmp_dir = PathBuf::from("/tmp/");

        Ok(Self {
            config_dir,
            data_dir,
            host_data_dir,
            tmp_dir,
        })
    }
}

// Tests removed due to unsafe env var usage
// Integration tests will verify configuration loading in practice
