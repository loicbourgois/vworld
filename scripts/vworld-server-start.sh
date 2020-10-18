$vworld_root_folder/scripts/vworld-build.sh || { exit 1; }
configuration_folder="$vworld_root_folder/configurations/$configuration_name" node $vworld_root_folder/scripts/generate-docker-run.js
chmod +x "$vworld_root_folder/configurations/$configuration_name/docker-run.sh"
$vworld_root_folder/configurations/${configuration_name}/docker-run.sh
