pwd_=$(pwd)
cd $vworld_root_folder/blooper
cargo fmt --verbose
cargo check || { cd $pwd_ ; exit 1; }
blooper_configuration=$(echo $(cat "$vworld_root_folder/blooper/configurations/${config}.json") | sed 's#\"#\\\"#g' )
blooper_configuration=$blooper_configuration cargo run --release
cd $pwd
