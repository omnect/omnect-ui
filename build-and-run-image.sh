# file used for local development

# local build and run
omnect_ui_version=$(toml get --raw Cargo.toml package.version)
rust_version="1.84.1-bookworm"
omnect_ui_port="1977"
centrifugo_port="8000"

docker build \
  --build-arg=DOCKER_NAMESPACE=omnectweucopsacr.azurecr.io \
  --build-arg=VERSION_RUST_CONTAINER="${rust_version}" \
  --build-arg=OMNECT_UI_BUILD_ARG="--features=mock" \
  -f Dockerfile \
  --progress=plain \
  -t omnect-ui-x86:"local_${omnect_ui_version}" .

# ensure presence of:
# /tmp/api.sock (normally created by a local instance of omnect-device-service)
# ./temp/cert.pem and ./temp/key.pem (certificate and key file)
docker run --rm \
  -v $(pwd)/temp:/cert \
  -v /tmp:/socket \
  -u $(id -u):$(id -g) \
  -e RUST_LOG=debug \
  -e UI_PORT=1977 \
  -e SOCKET_PATH=/socket/api.sock \
  -e LOGIN_USER=omnect-ui \
  -e LOGIN_PASSWORD=123 \
  -e CENTRIFUGO_CLIENT_TOKEN_HMAC_SECRET_KEY=my-token-secret-key \
  -e CENTRIFUGO_HTTP_API_KEY=my-api-key \
  -e CENTRIFUGO_CLIENT_ALLOWED_ORIGINS="https://$(hostname | tr [:upper:] [:lower:]):"${omnect_ui_port}" https://localhost:"${omnect_ui_port}"" \
  -e CENTRIFUGO_CHANNEL_WITHOUT_NAMESPACE_ALLOW_SUBSCRIBE_FOR_CLIENT=true \
  -e CENTRIFUGO_CHANNEL_WITHOUT_NAMESPACE_ALLOW_HISTORY_FOR_CLIENT=true \
  -e CENTRIFUGO_CHANNEL_WITHOUT_NAMESPACE_HISTORY_SIZE=1 \
  -e CENTRIFUGO_CHANNEL_WITHOUT_NAMESPACE_HISTORY_TTL=720h \
  -e CENTRIFUGO_HTTP_SERVER_TLS_ENABLED=true \
  -e CENTRIFUGO_ADMIN_ENABLED=true \
  -e CENTRIFUGO_ADMIN_PASSWORD=123 \
  -e CENTRIFUGO_ADMIN_SECRET=123 \
  -e DATA_DIR_PATH=$(pwd)/temp/data \
  -p "${omnect_ui_port}":"${omnect_ui_port}" \
  -p "${centrifugo_port}":"${centrifugo_port}" \
  omnect-ui-x86:"local_${omnect_ui_version}"
