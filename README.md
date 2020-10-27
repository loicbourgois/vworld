# VWorld

An artificial life simulation

## Quickstart

```bash
git clone git@github.com:loicbourgois/vworld.git
export vworld_root_folder=$(pwd)/vworld
$vworld_root_folder/scripts/demo.sh
```

## Other demos

```bash
$vworld_root_folder/scripts/fish.sh
```

## Deploy on [Scaleway Instances](https://www.scaleway.com/en/virtual-instances/)

```bash
export scaleway_secret_key=$(cat $HOME/.scaleway-vworld-secret-key)
export scaleway_organization_id=$(cat $HOME/.scaleway-vworld-organization-id)
$vworld_root_folder/scripts/scaleway-instance-deploy.sh
```
