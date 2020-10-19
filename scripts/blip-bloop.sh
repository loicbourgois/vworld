pwd_=$(pwd)
cd $vworld_root_folder/blip-bloop
cargo check || { cd $pwd_ ; exit 1; }
cargo run --release
cd $pwd_
