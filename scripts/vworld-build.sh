pwd_=$(pwd)
cd $vworld_root_folder/vworld-server
cargo check || /root/.cargo/bin/cargo check || { cd $pwd_ ; exit 1; }
cd $pwd_
DOCKER_BUILDKIT=1 docker build \
  --tag "vworld-server:latest" \
  --file $vworld_root_folder/vworld-server/Dockerfile \
  $vworld_root_folder/vworld-server
