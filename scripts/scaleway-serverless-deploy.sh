namespace_name="vworld"
region="fr-par"
docker_registry="rg.$region.scw.cloud"

log_action () {
  message=$1
  blue='\033[1;34m'
  no_color='\033[0m'
  echo "$blue$message$no_color"
}

echo "Welcome to VWorld"
echo ""
echo "You are about to:"
if [ "$upload_image" == "true" ]
then
  echo "  - create a new Scaleway container registry namespace '$namespace_name'"
  echo "  - upload a docker image 'vworld-server:latest' to this registry"
fi
echo "  - create a new Scaleway serverless namespace '$namespace_name'"
echo "  - create a new Scaleway serverless container '$namespace_name' based on 'vworld-server:latest'"
echo "  - deploy the serverless container '$namespace_name'"
echo ""
echo "To terminate the simulation and free up all resources at any time, run:"
echo "  \$vworld_root_folder/scripts/scaleway-cleanup.sh"
echo ""
read -p "Press enter to continue"

if [ "$upload_image" == "true" ]
then
  log_action "Building vworld-server:latest"
  $vworld_root_folder/scripts/vworld-build.sh

  log_action "Creating registry namespace"
  curl https://api.scaleway.com/registry/v1/regions/${region}/namespaces \
    --silent \
    --request POST \
    --header "x-auth-token: $scaleway_secret_key" \
    --data '{
      "name": "'$namespace_name'",
      "organization_id": "'$scaleway_organization_id'"
    }' | jq

  log_action "Login to docker registry $docker_registry"
  echo $scaleway_secret_key | docker login $docker_registry --username _ --password-stdin

  log_action "Pushing vworld-server:latest"
  docker tag vworld-server:latest $docker_registry/${namespace_name}/vworld-server:latest
  docker push $docker_registry/${namespace_name}/vworld-server:latest
fi

log_action "Creating container namespace"
container_namespace_id_response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/namespaces \
  --silent \
  --request POST \
  --header "x-auth-token: $scaleway_secret_key" \
  --data '{
    "name": "'$namespace_name'",
    "organization_id": "'$scaleway_organization_id'"
  }')
container_namespace_id=$(echo $container_namespace_id_response | jq -r .id)
container_namespace_registry_id=$(echo $container_namespace_id_response | jq -r .registry_namespace_id)
status="nope"
while [ "$status" != "ready" ]
do
  response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/namespaces/${container_namespace_id} \
    --silent \
    --request GET \
    --header "x-auth-token: $scaleway_secret_key")
  #echo $response | jq
  status=$(echo $response | jq -r .status)
  echo "status: $status"
  if [ "$status" == "null" ]
  then
    echo $container_namespace_id_response | jq
    echo $response | tr -d '\n' | jq
    exit 0
  fi
  sleep 1
done

log_action "Generating chunk configuration"
configuration_name=$configuration
configuration_folder="$vworld_root_folder/configurations/$configuration_name"
configuration_folder=$configuration_folder \
  x=$x \
  y=$y \
  node $vworld_root_folder/scripts/generate-chunk-configuration.js || { exit 1; }

log_action "Creating container"
vworld_chunk_configuration=$(echo $(cat "$configuration_folder/$x:$y.chunk.json") | sed 's#\"#\\\"#g' )
vworld_address="0.0.0.0"
vworld_port="8080"
domain_name="vworld-$configuration_name-$x-$y"
data='{
  "namespace_id": "'$container_namespace_id'",
  "name": "'$namespace_name'",
  "environment_variables": {
    "vworld_address": "'$vworld_address'",
    "vworld_port": "'$vworld_port'",
    "vworld_chunk_configuration": "'$vworld_chunk_configuration'"
  },
  "min_scale": 1,
  "max_scale": 1,
  "memory_limit": 2048,
  "timeout": 10000,
  "registry_image": "'${docker_registry}/${namespace_name}/vworld-server:latest'",
  "max_concurrency": 1
}'
container_post_response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/containers \
  --silent \
  --request POST \
  --header "x-auth-token: $scaleway_secret_key" \
  --data "$(echo $data)" | jq)
echo $container_post_response | jq
container_id=$(echo $container_post_response | jq --raw-output .id)
status="nope"
while [ "$status" != "created" ]
do
  response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/containers/${container_id} \
    --silent \
    --request GET \
    --header "x-auth-token: $scaleway_secret_key")
  #echo $response | jq
  status=$(echo $response | jq -r .status)
  echo "status: $status"
  sleep 1
  if [ "$status" == "null" ]
  then
    echo $response | tr -d '\n' | jq
    exit 0
  fi
done

log_action "Deploying container"
deploy_container_response=$(curl "https://api.scaleway.com/functions/v1alpha2/regions/${region}/containers/${container_id}/deploy" \
  --silent \
  --request POST \
  --header "X-Auth-Token: $scaleway_secret_key" \
  --data "{}")
# echo $deploy_container_response | jq
container_id=$(echo $deploy_container_response | jq --raw-output .id)
status="nope"
while [ "$status" != "ready" ]
do
  response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/containers/${container_id} \
    --silent \
    --request GET \
    --header "x-auth-token: $scaleway_secret_key")
  #echo $response
  #echo $response | tr -d '\n' | jq
  status=$(echo $response | tr -d '\n' | jq -r .status)
  echo "status: $status"
  if [ "$status" == "ready" ]
  then
    endpoint=$(echo $response | tr -d '\n' | jq --raw-output .endpoint)
    echo "endpoint: $endpoint"
    exit 0
  fi
  if [ "$status" == "error" ]
  then
    echo $response | tr -d '\n' | jq
    exit 0
  fi
  sleep 1
done
