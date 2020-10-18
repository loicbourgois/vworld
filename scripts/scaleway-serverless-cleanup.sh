namespace_name="vworld"
region="fr-par"
blue='\033[1;34m'
no_color='\033[0m'
log_action () {
  message=$1
  blue='\033[1;34m'
  no_color='\033[0m'
  echo "$blue$message$no_color"
}

echo "${blue}Getting registry namespace id for '$namespace_name'${no_color}"
registry_namespace_response=$(curl https://api.scaleway.com/registry/v1/regions/${region}/namespaces \
  --silent \
  --request GET \
  --header "x-auth-token: $scaleway_secret_key" \
  --data '{
    "name": "'$namespace_name'",
    "organization_id": "'$scaleway_organization_id'"
  }')
registry_namespace_id=$(echo $registry_namespace_response | jq --raw-output .namespaces[0].id)
echo "registry_namespace_id: $registry_namespace_id"

if [ "$delete_image" == "true" ]
then
  log_action "Deleting registry namespace '$namespace_name'"
  response_delete=$(curl https://api.scaleway.com/registry/v1/regions/$region/namespaces/$registry_namespace_id \
    --silent \
    --request DELETE \
    --header "x-auth-token: $scaleway_secret_key" | jq)
  delete_id=$(echo $response_delete | jq -r .id)
  status="deleting"
  while [ "$status" == "deleting" ]
  do
    response_deleting=$(curl https://api.scaleway.com/registry/v1/regions/$region/namespaces/$registry_namespace_id \
      --silent \
      --request GET \
      --header "x-auth-token: $scaleway_secret_key" | jq)
    #echo $response | jq
    status=$(echo $response_deleting | jq -r .status)
    echo "status: $status (#)"
    sleep 1
  done
fi

echo "${blue}Getting container id${no_color}"
data='{
  "name": "'$namespace_name'"
}'
container_get_response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/containers \
  --silent \
  --request GET \
  --header "x-auth-token: $scaleway_secret_key" \
  --data "$(echo $data)")
container_id=$(echo $container_get_response | tr -d '\n' | jq --raw-output .containers[0].id)
echo "container_id: $container_id"

echo "${blue}Deleting container $container_id${no_color}"
container_delete_response=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/containers/${container_id} \
  --silent \
  --request DELETE \
  --header "x-auth-token: $scaleway_secret_key")
echo $container_delete_response

echo "${blue}Getting container namespace id for '$namespace_name'${no_color}"
container_namespace_id=$(curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/namespaces \
  --silent \
  --request GET \
  --header "x-auth-token: $scaleway_secret_key" \
  --data '{
    "name": "'$namespace_name'",
    "organization_id": "'$scaleway_organization_id'"
  }' | jq --raw-output .namespaces[0].id
)

echo "${blue}Deleting container namespace '$namespace_name'${no_color}"
curl https://api.scaleway.com/functions/v1alpha2/regions/${region}/namespaces/${container_namespace_id} \
  --silent \
  --request DELETE \
  --header "x-auth-token: $scaleway_secret_key" | jq
