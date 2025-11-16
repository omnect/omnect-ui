#!/bin/bash
# file used for local development

set -e

# Default configuration
DEVICE_HOST="${DEVICE_HOST:-192.168.0.98}"
DEVICE_USER="${DEVICE_USER:-omnect}"
DEVICE_PORT="${DEVICE_PORT:-1977}"
IMAGE_TAG="${IMAGE_TAG:-$(whoami)}"
IMAGE_ARCH="${IMAGE_ARCH:-arm64}"
IMAGE_NAME="omnectshareddevacr.azurecr.io/omnect-ui:${IMAGE_TAG}"
IMAGE_TAR="/tmp/omnect-ui-${IMAGE_ARCH}.tar"

usage() {
  cat << EOF
Usage: $0 [OPTIONS]

Build Docker image for omnect-ui and optionally deploy to device.

OPTIONS:
  --deploy              Deploy the image to the target device after building
  --push                Push the image to the registry after building
  --arch <arch>         Target architecture (default: $IMAGE_ARCH)
  --host <hostname>     Target device hostname or IP (default: $DEVICE_HOST)
  --user <username>     SSH user for target device (default: $DEVICE_USER)
  --port <port>         UI port on target device (default: $DEVICE_PORT)
  --tag <tag>           Docker image tag (default: \$(whoami))
  --help                Show this help message

ENVIRONMENT VARIABLES:
  DEVICE_HOST           Target device hostname or IP (default: 192.168.0.98)
  DEVICE_USER           SSH user for target device (default: omnect)
  DEVICE_PORT           UI port on target device (default: 1977)
  IMAGE_TAG             Docker image tag (default: \$(whoami))
  IMAGE_ARCH            Target architecture (default: arm64)

EXAMPLES:
  $0                                    # Build only (arm64)
  $0 --arch amd64                       # Build for amd64
  $0 --deploy                           # Build and deploy to default device
  $0 --push                             # Build and push to registry
  $0 --push --deploy                    # Build, push to registry, and deploy
  $0 --deploy --host 192.168.1.100      # Build and deploy to specific device
  $0 --tag v1.2.0 --deploy              # Build with custom tag and deploy
  $0 --deploy --port 8080               # Build and deploy with custom port
  IMAGE_TAG=test $0 --deploy            # Build with env var tag and deploy
EOF
}

# Parse command line arguments
DEPLOY=false
PUSH=false
while [[ $# -gt 0 ]]; do
  case $1 in
    --deploy)
      DEPLOY=true
      shift
      ;;
    --push)
      PUSH=true
      shift
      ;;
    --arch)
      IMAGE_ARCH="$2"
      IMAGE_TAR="/tmp/omnect-ui-${IMAGE_ARCH}.tar"
      shift 2
      ;;
    --host)
      DEVICE_HOST="$2"
      shift 2
      ;;
    --user)
      DEVICE_USER="$2"
      shift 2
      ;;
    --port)
      DEVICE_PORT="$2"
      shift 2
      ;;
    --tag)
      IMAGE_TAG="$2"
      IMAGE_NAME="omnectshareddevacr.azurecr.io/omnect-ui:${IMAGE_TAG}"
      shift 2
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "Error: Unknown option $1"
      usage
      exit 1
      ;;
  esac
done

# local build
omnect_ui_version=$(toml get --raw Cargo.toml workspace.package.version)

echo "Building ${IMAGE_ARCH} image: $IMAGE_NAME"

# Setup QEMU for cross-architecture builds if needed
if [[ "$IMAGE_ARCH" != "$(uname -m)" ]]; then
  docker run --rm --privileged omnectweucopsacr.azurecr.io/mlilien/qemu-user-static:8.1.2 --reset -p yes
fi

docker buildx build \
  --platform "linux/${IMAGE_ARCH}" \
  --load \
  -f Dockerfile . \
  -t "$IMAGE_NAME"

# Push to registry if requested
if [[ "$PUSH" == "true" ]]; then
  echo "Pushing image to registry..."
  docker push "$IMAGE_NAME"
  echo "Image pushed successfully: $IMAGE_NAME"
fi

# Deploy to device if requested
if [[ "$DEPLOY" == "true" ]]; then
  echo "Saving Docker image to $IMAGE_TAR..."
  docker save "$IMAGE_NAME" -o "$IMAGE_TAR"

  echo "Copying image to device $DEVICE_HOST..."
  scp "$IMAGE_TAR" "${DEVICE_USER}@${DEVICE_HOST}:/tmp/"

  echo "Loading image on device and restarting container..."
  ssh "${DEVICE_USER}@${DEVICE_HOST}" << EOF
    set -e

    # Check required directories exist
    echo "Checking required directories..."
    for dir in /run/omnect-device-service /var/lib/omnect-ui /etc/systemd/network; do
      if [ ! -d "\$dir" ]; then
        echo "ERROR: Required directory \$dir does not exist on device"
        exit 1
      fi
    done
    echo "All required directories exist"

    sudo iotedge system stop
    sudo docker container rm -f omnect-ui
    sudo docker image rm -f ${IMAGE_NAME}
    sudo docker load -i /tmp/omnect-ui-${IMAGE_ARCH}.tar
    rm /tmp/omnect-ui-${IMAGE_ARCH}.tar
    sleep 5
    sudo iotedge system restart

EOF

  echo "Cleaning up local tar file..."
  rm -f "$IMAGE_TAR"

  echo "Deployment complete! Access at https://${DEVICE_HOST}:${DEVICE_PORT}"
else
  echo "Image built successfully: $IMAGE_NAME"
  echo "To deploy to device, run: $0 --deploy"
  echo "Run '$0 --help' for more options."
fi
