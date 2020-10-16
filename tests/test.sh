pwd_=$(pwd)
cd $vworld_root_folder/tests
cargo check || { exit 1; }
cargo run || { exit 1; }
./test
cd $pwd_
