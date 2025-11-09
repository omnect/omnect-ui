use anyhow::{Context, Result};
use std::{env, path::PathBuf, sync::OnceLock};
use uuid::Uuid;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

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

    /// IoT Edge workload API configuration
    #[cfg_attr(feature = "mock", allow(dead_code))]
    pub iot_edge: IoTEdgeConfig,

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
#[cfg_attr(feature = "mock", allow(dead_code))]
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
    /// Get or load the application configuration
    ///
    /// Returns a reference to the cached configuration. On first call, it loads
    /// and validates all configuration from environment variables. Subsequent
    /// calls return the cached instance.
    ///
    /// # Panics
    /// Panics if configuration loading fails. This is intentional as the
    /// application cannot function without valid configuration.
    pub fn get() -> &'static Self {
        APP_CONFIG.get_or_init(|| {
            Self::load_internal().expect("Failed to load application configuration")
        })
    }

    /// Internal function to load and validate all configuration from environment variables
    ///
    /// This should only be called once via get(). It validates all
    /// required environment variables and returns an error if any are missing
    /// or invalid.
    fn load_internal() -> Result<Self> {
        // Validate critical paths exist before proceeding (skip in test/mock mode)
        #[cfg(not(any(test, feature = "mock")))]
        if !std::fs::exists("/data").is_ok_and(|ok| ok) {
            anyhow::bail!("failed to find required data directory: /data is missing");
        }

        let ui = UiConfig::load()?;
        let centrifugo = CentrifugoConfig::load()?;
        let keycloak = KeycloakConfig::load()?;
        let device_service = DeviceServiceConfig::load()?;
        let certificate = CertificateConfig::load()?;
        let iot_edge = IoTEdgeConfig::load()?;
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
}

impl UiConfig {
    fn load() -> Result<Self> {
        let port = env::var("UI_PORT")
            .unwrap_or_else(|_| "443".to_string())
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
    fn load() -> Result<Self> {
        let module_id = env::var("IOTEDGE_MODULEID").unwrap_or_else(|_| "test-module".to_string());
        let module_generation_id =
            env::var("IOTEDGE_MODULEGENERATIONID").unwrap_or_else(|_| "1".to_string());
        let api_version =
            env::var("IOTEDGE_APIVERSION").unwrap_or_else(|_| "2021-12-07".to_string());
        let workload_uri = env::var("IOTEDGE_WORKLOADURI")
            .unwrap_or_else(|_| "unix:///var/run/iotedge/workload.sock".to_string());

        Ok(Self {
            module_id,
            module_generation_id,
            api_version,
            workload_uri,
        })
    }
}

impl PathConfig {
    fn load() -> Result<Self> {
        // In test mode, use temp directory as default to avoid /data requirement
        #[cfg(test)]
        let default_config = std::env::temp_dir()
            .join("omnect-test-config")
            .display()
            .to_string();
        #[cfg(not(test))]
        let default_config = "/data/config".to_string();

        let config_dir: PathBuf = env::var("CONFIG_PATH").unwrap_or(default_config).into();

        // Ensure config directory exists (skip in test/mock mode as it may not have permissions)
        #[cfg(not(any(test, feature = "mock")))]
        if !std::fs::exists(&config_dir).is_ok_and(|ok| ok) {
            std::fs::create_dir_all(&config_dir)
                .context("failed to create config directory")?;
        }

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
