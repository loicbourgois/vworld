# VWorld

An artificial life simulation

## Quickstart

```bash
git clone git@github.com:loicbourgois/vworld.git
export vworld_root_folder=$(pwd)/vworld
$vworld_root_folder/scripts/demo.sh
```

## Deploy on [Scaleway Instances](https://www.scaleway.com/en/virtual-instances/)

```bash
export scaleway_secret_key=$(cat $HOME/.scaleway-vworld-secret-key)
export scaleway_organization_id=$(cat $HOME/.scaleway-vworld-organization-id)
$vworld_root_folder/scripts/scaleway-instance-deploy.sh
```

To terminate the simulation and free up all resources:
```bash
$vworld_root_folder/scripts/scaleway-instance-cleanup.sh
```


## Deploy on [Scaleway Serverless](https://www.scaleway.com/en/elements/)

```bash
export scaleway_secret_key=$(cat $HOME/.scaleway-vworld-secret-key)
export scaleway_organization_id=$(cat $HOME/.scaleway-vworld-organization-id)
upload_image="true" configuration="demo" x="0" y="0" $vworld_root_folder/scripts/scaleway-serverless-deploy.sh
```

To terminate the simulation and free up all resources:
```bash
delete_image="true" $vworld_root_folder/scripts/scaleway-serverless-cleanup.sh
```
