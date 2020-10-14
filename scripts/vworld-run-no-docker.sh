pwd_=$(pwd)
cd $vworld_root_folder/vworld-server
port="10000" address="127.0.0.1" configuration='{"particles":[{"type_":"sun","x":0.5,"y":0.5}],"x":0,"y":0}' cargo run
cd $pwd_
