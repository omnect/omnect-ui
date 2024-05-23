# local build
omnect_ui_version=$(toml get --raw Cargo.toml package.version)
rust_version="1.78.0-bookworm"

docker build \
  --no-cache \
  --build-arg=DOCKER_NAMESPACE=omnectweucopsacr.azurecr.io \
  --build-arg=VERSION_RUST_CONTAINER="${rust_version}" \
  -f Dockerfile \
  --progress=plain \
  -t omnect-ui:"local_${omnect_ui_version}" .

  docker run --rm \
  -v "$(pwd)":/cert \
  -e RUST_LOG=debug \
  -e CENTRIFUGO_TOKEN_HMAC_SECRET_KEY=my-test-key \
  -e LOGIN_USER=omnect-ui \
  -e LOGIN_PASSWORD=123 \
  -p 1977:1977 \
  -p 8000:8000 \
  omnect-ui:"local_${omnect_ui_version}"