#![cfg_attr(feature = "mock", allow(dead_code, unused_imports))]

use crate::{
    config::AppConfig,
    http_client::{HttpClientFactory, handle_http_response},
    omnect_device_service_client::DeviceServiceClient,
};
use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

#[derive(Serialize)]
struct CreateCertPayload {
    #[serde(rename = "commonName")]
    common_name: String,
}

#[derive(Debug, Deserialize)]
struct PrivateKey {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_name: String,
    bytes: String,
}

#[derive(Debug, Deserialize)]
struct CreateCertResponse {
    #[serde(rename = "privateKey")]
    private_key: PrivateKey,
    certificate: String,
    #[allow(dead_code)]
    expiration: String,
}

#[cfg(feature = "mock")]
pub async fn create_module_certificate<T>(_service_client: &T) -> Result<()>
where
    T: DeviceServiceClient,
{
    Ok(())
}

#[cfg(not(feature = "mock"))]
pub async fn create_module_certificate<T>(service_client: &T) -> Result<()>
where
    T: DeviceServiceClient,
{
    info!("create module certificate");

    let iot_edge = &AppConfig::get().iot_edge;

    let payload = CreateCertPayload {
        common_name: service_client.ip_address().await?,
    };

    let path = format!(
        "/modules/{}/genid/{}/certificate/server?api-version={}",
        iot_edge.module_id, iot_edge.module_generation_id, iot_edge.api_version
    );

    // Create a client for the IoT Edge workload socket
    let client = HttpClientFactory::workload_client(&iot_edge.workload_uri)?;

    let url = format!("http://localhost{}", path);
    info!("POST {url} (IoT Edge workload API)");

    let res = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("failed to send certificate request to IoT Edge workload API")?;

    let body = handle_http_response(res, "certificate request").await?;
    let response: CreateCertResponse =
        serde_json::from_str(&body).context("failed to parse CreateCertResponse")?;

    let config = AppConfig::get();
    let mut file =
        File::create(&config.certificate.cert_path).context("failed to create cert file")?;
    file.write_all(response.certificate.as_bytes())
        .context("failed to write certificate to file")?;

    let mut file =
        File::create(&config.certificate.key_path).context("failed to create key file")?;
    file.write_all(response.private_key.bytes.as_bytes())
        .context("failed to write private key to file")
}
