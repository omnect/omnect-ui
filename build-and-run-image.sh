# local build
omnect_ui_version=$(toml get --raw Cargo.toml package.version)
rust_version="1.78.0-bookworm"

docker build \
  --build-arg=DOCKER_NAMESPACE=omnectweucopsacr.azurecr.io \
  --build-arg=VERSION_RUST_CONTAINER="${rust_version}" \
  -f Dockerfile \
  --progress=plain \
  -t omnect-ui:"local_${omnect_ui_version}" .

docker run --rm \
  -v "$(pwd)":/cert \
  --mount type=bind,source=/tmp/api.sock,target=/run/omnect-device-service/api.sock \
  -u $(id -u):$(id -g) \
  -e RUST_LOG=debug \
  -e CENTRIFUGO_TOKEN_HMAC_SECRET_KEY=my-token-secret-key \
  -e CENTRIFUGO_API_KEY=my-api-key \
  -e CENTRIFUGO_ADMIN=true \
  -e CENTRIFUGO_ADMIN_PASSWORD=123 \
  -e CENTRIFUGO_ADMIN_SECRET=123 \
  -e CENTRIFUGO_LOG_LEVEL=debug \
  -e LOGIN_USER=omnect-ui \
  -e LOGIN_PASSWORD=123 \
  -p 1977:1977 \
  -p 8000:8000 \
  omnect-ui:"local_${omnect_ui_version}"