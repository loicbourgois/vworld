# VWorld - developer's guide

## Useful scripts

```bash
export vworld_root_folder=$HOME/github/vworld
$vworld_root_folder/scripts/blooper.sh

configuration_name="demo" $vworld_root_folder/scripts/vworld-server-start.sh

$vworld_root_folder/scripts/setup-server.sh

$vworld_root_folder/tests/test.sh
$vworld_root_folder/scripts/check.sh
$vworld_root_folder/scripts/front.sh
$vworld_root_folder/scripts/vworld-build.sh
$vworld_root_folder/scripts/vworld-run-no-docker.sh
configuration_name="demo" x="0" y="0" $vworld_root_folder/scripts/vworld-server-start-singlechunk.sh
```

## Blooper

### Requirements

#### Vulkan - macos

```
curl "https://sdk.lunarg.com/sdk/download/1.2.154.0/mac/vulkansdk-macos-1.2.154.0.dmg?Human=true" --output vulkansdk-macos-1.2.154.0.dmg
open vulkansdk-macos-1.2.154.0.dmg
python vulkansdk-macos-1.2.154.0/install_vulkan.py
```

```
brew install cmake
```
