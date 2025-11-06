use omnect_ui::http_client::HttpClientFactory;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

#[derive(Serialize, Deserialize, Debug)]
struct CreateCertPayload {
    #[serde(rename = "commonName")]
    common_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrivateKey {
    #[serde(rename = "type")]
    type_name: String,
    bytes: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateCertResponse {
    #[serde(rename = "privateKey")]
    private_key: PrivateKey,
    certificate: String,
    expiration: String,
}

async fn start_mock_workload_server(
    socket_path: PathBuf,
    ready_tx: oneshot::Sender<()>,
) -> std::io::Result<()> {
    let listener = UnixListener::bind(&socket_path)?;

    // Signal that the server is ready
    let _ = ready_tx.send(());

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut reader = BufReader::new(&mut stream);
            let mut headers = Vec::new();
            let mut content_length = 0;

            // Read HTTP headers
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }

                if line.trim().is_empty() {
                    break;
                }

                if line.to_lowercase().starts_with("content-length:")
                    && let Some(len_str) = line.split(':').nth(1)
                {
                    content_length = len_str.trim().parse().unwrap_or(0);
                }

                headers.push(line);
            }

            // Read the request body if present
            let mut body = vec![0u8; content_length];
            if content_length > 0 && reader.read_exact(&mut body).await.is_err() {
                return;
            }

            // Parse the payload to extract common_name
            let common_name =
                if let Ok(payload) = serde_json::from_slice::<CreateCertPayload>(&body) {
                    payload.common_name
                } else {
                    "unknown".to_string()
                };

            // Create a mock response
            let response = CreateCertResponse {
                private_key: PrivateKey {
                    type_name: "key".to_string(),
                    bytes: format!(
                        "-----BEGIN PRIVATE KEY-----\nMOCK_KEY_FOR_{}\n-----END PRIVATE KEY-----",
                        common_name
                    ),
                },
                certificate: format!(
                    "-----BEGIN CERTIFICATE-----\nMOCK_CERT_FOR_{}\n-----END CERTIFICATE-----",
                    common_name
                ),
                expiration: "2025-12-31T23:59:59Z".to_string(),
            };

            let response_body = serde_json::to_string(&response).unwrap();
            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );

            let _ = stream.write_all(http_response.as_bytes()).await;
        });
    }
}

#[tokio::test]
async fn test_workload_client_integration_success() {
    // Create a temporary directory for the Unix socket
    let temp_dir = TempDir::new().expect("failed to create temp directory");
    let socket_path = temp_dir.path().join("workload.sock");
    let socket_path_clone = socket_path.clone();

    // Create a oneshot channel for server ready signal
    let (ready_tx, ready_rx) = oneshot::channel();

    // Start the mock server in the background
    let server_handle = tokio::spawn(async move {
        let _ = start_mock_workload_server(socket_path_clone, ready_tx).await;
    });

    // Wait for the server to be ready
    ready_rx.await.expect("server failed to start");

    // Create the workload client using the factory
    let workload_uri = format!("unix://{}", socket_path.display());
    let client = HttpClientFactory::workload_client(&workload_uri)
        .expect("failed to create workload client");

    // Make a request to the mock workload API
    let payload = CreateCertPayload {
        common_name: "test-module".to_string(),
    };

    let url = "http://localhost/modules/testmodule/genid/1234/certificate/server";
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .expect("failed to send request");

    // Verify the response
    assert!(response.status().is_success());

    let cert_response: CreateCertResponse =
        response.json().await.expect("failed to parse response");

    assert!(
        cert_response
            .certificate
            .contains("MOCK_CERT_FOR_test-module")
    );
    assert!(
        cert_response
            .private_key
            .bytes
            .contains("MOCK_KEY_FOR_test-module")
    );

    // Clean up
    server_handle.abort();
}

// Integration tests for unix_socket_client
async fn start_mock_unix_socket_server(
    socket_path: PathBuf,
    ready_tx: oneshot::Sender<()>,
) -> std::io::Result<()> {
    let listener = UnixListener::bind(&socket_path)?;

    // Signal that the server is ready
    let _ = ready_tx.send(());

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut reader = BufReader::new(&mut stream);
            let mut _headers = Vec::new();

            // Read HTTP headers
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }

                if line.trim().is_empty() {
                    break;
                }

                _headers.push(line);
            }

            // Simple mock response
            let response_body = r#"{"status":"ok","message":"test response"}"#;
            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );

            let _ = stream.write_all(http_response.as_bytes()).await;
        });
    }
}

#[tokio::test]
async fn test_unix_socket_client_integration_success() {
    // Create a temporary directory for the Unix socket
    let temp_dir = TempDir::new().expect("failed to create temp directory");
    let socket_path = temp_dir.path().join("test.sock");
    let socket_path_clone = socket_path.clone();

    // Create a oneshot channel for server ready signal
    let (ready_tx, ready_rx) = oneshot::channel();

    // Start the mock server in the background
    let server_handle = tokio::spawn(async move {
        let _ = start_mock_unix_socket_server(socket_path_clone, ready_tx).await;
    });

    // Wait for the server to be ready
    ready_rx.await.expect("server failed to start");

    // Create the unix socket client using the factory
    let client = HttpClientFactory::unix_socket_client(&socket_path)
        .expect("failed to create unix socket client");

    // Make a request to the mock server
    let url = "http://localhost/test";
    let response = client
        .get(url)
        .send()
        .await
        .expect("failed to send request");

    // Verify the response
    assert!(response.status().is_success());

    let body = response.text().await.expect("failed to read response body");
    assert!(body.contains("test response"));

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_unix_socket_client_integration_post_request() {
    // Create a temporary directory for the Unix socket
    let temp_dir = TempDir::new().expect("failed to create temp directory");
    let socket_path = temp_dir.path().join("test-post.sock");
    let socket_path_clone = socket_path.clone();

    // Create a oneshot channel for server ready signal
    let (ready_tx, ready_rx) = oneshot::channel();

    // Start the mock server in the background
    let server_handle = tokio::spawn(async move {
        let _ = start_mock_unix_socket_server(socket_path_clone, ready_tx).await;
    });

    // Wait for the server to be ready
    ready_rx.await.expect("server failed to start");

    // Create the unix socket client using the factory
    let client = HttpClientFactory::unix_socket_client(&socket_path)
        .expect("failed to create unix socket client");

    // Make a POST request with JSON payload
    #[derive(Serialize)]
    struct TestPayload {
        name: String,
        value: i32,
    }

    let payload = TestPayload {
        name: "test".to_string(),
        value: 42,
    };

    let url = "http://localhost/api/data";
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .expect("failed to send request");

    // Verify the response
    assert!(response.status().is_success());

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_unix_socket_client_integration_multiple_requests() {
    // Create a temporary directory for the Unix socket
    let temp_dir = TempDir::new().expect("failed to create temp directory");
    let socket_path = temp_dir.path().join("test-multi.sock");
    let socket_path_clone = socket_path.clone();

    // Create a oneshot channel for server ready signal
    let (ready_tx, ready_rx) = oneshot::channel();

    // Start the mock server in the background
    let server_handle = tokio::spawn(async move {
        let _ = start_mock_unix_socket_server(socket_path_clone, ready_tx).await;
    });

    // Wait for the server to be ready
    ready_rx.await.expect("server failed to start");

    // Create the unix socket client using the factory
    let client = HttpClientFactory::unix_socket_client(&socket_path)
        .expect("failed to create unix socket client");

    // Make multiple requests to ensure the client can be reused
    for i in 0..3 {
        let url = format!("http://localhost/test/{}", i);
        let response = client
            .get(&url)
            .send()
            .await
            .expect("failed to send request");

        assert!(response.status().is_success());
    }

    // Clean up
    server_handle.abort();
}
