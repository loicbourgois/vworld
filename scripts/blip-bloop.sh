pwd_=$(pwd)
cd $vworld_root_folder/blip-bloop
cargo check || { cd $pwd_ ; exit 1; }
blip_bloop_port="10001" blip_bloop_address="127.0.0.1" cargo run --release
cd $pwd_
