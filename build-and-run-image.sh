# local build and run
omnect_ui_version=$(toml get --raw Cargo.toml package.version)
rust_version="1.78.0-bookworm"

docker build \
  --build-arg=DOCKER_NAMESPACE=omnectweucopsacr.azurecr.io \
  --build-arg=VERSION_RUST_CONTAINER="${rust_version}" \
  -f Dockerfile \
  --progress=plain \
  -t omnect-ui:"local_${omnect_ui_version}" .

# /tmp/api.sock used by omnect-device-service
# ./temp/device_id_cert.pem and temp/device_id_cert_key.pem must exist
docker run --rm \
  -v $(pwd)/temp:/temp \
  --mount type=bind,source=/tmp/api.sock,target=/temp/api.sock \
  -u $(id -u):$(id -g) \
  -e RUST_LOG=debug \
  -e SOCKET_PATH=/temp/api.sock \
  -e SSL_CERT_PATH=/temp/device_id_cert.pem \
  -e SSL_KEY_PATH=/temp/device_id_cert_key.pem \
  -e LOGIN_USER=omnect-ui \
  -e LOGIN_PASSWORD=123 \
  -e CENTRIFUGO_TOKEN_HMAC_SECRET_KEY=my-token-secret-key \
  -e CENTRIFUGO_API_KEY=my-api-key \
  -e CENTRIFUGO_ALLOWED_ORIGINS="https://$(hostname | tr [:upper:] [:lower:]):1977 https://localhost:1977" \
  -e CENTRIFUGO_ALLOW_SUBSCRIBE_FOR_CLIENT=true \
  -e CENTRIFUGO_ALLOW_HISTORY_FOR_CLIENT=true \
  -e CENTRIFUGO_HISTORY_SIZE=1 \
  -e CENTRIFUGO_HISTORY_TTL=720h \
  -e CENTRIFUGO_TLS=true \
  -e CENTRIFUGO_TLS_CERT=/temp/device_id_cert.pem \
  -e CENTRIFUGO_TLS_KEY=/temp/device_id_cert_key.pem \
  -e CENTRIFUGO_ADMIN=true \
  -e CENTRIFUGO_ADMIN_PASSWORD=123 \
  -e CENTRIFUGO_ADMIN_SECRET=123 \
  -p 1977:1977 \
  -p 8000:8000 \
  omnect-ui:"local_${omnect_ui_version}"
