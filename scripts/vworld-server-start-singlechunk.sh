pwd_=$(pwd)
if [ "$mode" == "release" ]; then
    release="--release"
else
    release=""
fi
cd $vworld_root_folder/vworld-server
cargo check || { cd $pwd_ ; exit 1; }
configuration_folder="$vworld_root_folder/configurations/$configuration_name"
configuration_folder=$configuration_folder x=$x y=$y node $vworld_root_folder/scripts/generate-chunk-configuration.js || { exit 1; }
chunk_configuration="$(cat "$configuration_folder/$x:$y.chunk.json")"
chunk_configuration=$(echo $(cat "$configuration_folder/$x:$y.chunk.json") | sed 's#\"#\\\"#g' )
vworld_port="10001" vworld_address="127.0.0.1" vworld_chunk_configuration=$chunk_configuration cargo run $release
cd $pwd_
