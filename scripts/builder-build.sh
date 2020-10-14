DOCKER_BUILDKIT=1 docker build \
  --tag "vworld-builder:latest" \
  --file $vworld_root_folder/builder/Dockerfile \
  $vworld_root_folder
