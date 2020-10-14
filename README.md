# VWorld

An artificial life simulation

## Quickstart

```bash
git clone git@github.com:loicbourgois/vworld.git
export vworld_root_folder=$(pwd)/vworld
$vworld_root_folder/scripts/demo.sh
```

## Deploy on [Scaleway Elements](https://www.scaleway.com/en/elements/)

```bash
export scaleway_secret_key=$(cat $HOME/.scaleway-vworld-secret-key)
export scaleway_organization_id=$(cat $HOME/.scaleway-vworld-organization-id)
upload_image="true" configuration="demo" x="0" y="0" $vworld_root_folder/scripts/scaleway-deploy.sh
```

To terminate the simulation and free up all resources at any time:
```bash
delete_image="true" $vworld_root_folder/scripts/scaleway-cleanup.sh
```
