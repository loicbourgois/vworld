region="fr-par"
zone="fr-par-1"
image_uuid="77882134-b04d-456a-a891-be2995aae7d8"
server_type="X64-60GB"
server_type="START1-L"
server_type="GP1-M"
log_action () {
  message=$1
  blue='\033[1;34m'
  no_color='\033[0m'
  echo "$blue$message$no_color"
}


log_action "Listing Scaleway instances images"
data='{
  "name": "Debian Buster"
}'
response=$(curl https://api.scaleway.com/instance/v1/zones/${zone}/images \
  --silent \
  --request GET \
  --data "$(echo $data)" \
  --header "x-auth-token: $scaleway_secret_key")
echo $response | jq --raw-output '.images[] | .id + " " + .arch + " " + .name'


log_action "Listing Scaleway instances server types"
response=$(curl https://api.scaleway.com/instance/v1/zones/${zone}/products/servers \
  --silent \
  --request GET \
  --header "x-auth-token: $scaleway_secret_key")
echo $response | jq --raw-output '.servers | keys[] as $k |   "\($k), \(.[$k] | (.ncpus|tostring) + " " +   (.monthly_price|tostring))" '


log_action "Creating Scaleway Instance"
data='{
  "name": "vworld",
  "commercial_type": "'$server_type'",
  "image": "'$image_uuid'",
  "dynamic_ip_required": true,
  "organization": "'$scaleway_organization_id'"
}'
response=$(curl https://api.scaleway.com/instance/v1/zones/${zone}/servers \
  --silent \
  --request POST \
  --data "$(echo $data)" \
  --header "Content-Type: application/json" \
  --header "x-auth-token: $scaleway_secret_key")
echo $response | jq
server_id=$(echo $response | jq --raw-output '.server.id')


log_action "Starting instance"
data='{
  "action": "poweron"
}'
response=$(curl https://api.scaleway.com/instance/v1/zones/${zone}/servers/${server_id}/action \
  --silent \
  --request POST \
  --data "$(echo $data)" \
  --header "Content-Type: application/json" \
  --header "x-auth-token: $scaleway_secret_key")
echo $response | jq


log_action "Waiting for instance to be on"
state="none"
while [ "$state" != "running" ]
do
  sleep 2
  response=$(curl https://api.scaleway.com/instance/v1/zones/${zone}/servers/${server_id} \
    --silent \
    --request GET \
    --header "Content-Type: application/json" \
    --header "x-auth-token: $scaleway_secret_key")
  state=$(echo $response | jq -r .server.state)
  echo "state: $state"
  if [ "$state" == "null" ]
  then
    echo $response | jq
    exit 0
  fi
done
public_ip=$(echo $response | jq -r .server.public_ip.address)
echo "public_ip: $public_ip"


log_action "Waiting for port 22"
nc -zw1000 $public_ip 22
nc -zw100 $public_ip 22


log_action "Starting vworld"
git_current_branch=$(git rev-parse --abbrev-ref HEAD)
git_branch="$git_current_branch" vworld_host="$public_ip" vworld_config="demo" $vworld_root_folder/scripts/setup-server.sh || { exit 1; }


echo ""
log_action "Server ready!"
echo "restart vworld server with ansible"
echo "  git_branch='$git_current_branch' vworld_host='$public_ip' vworld_config='demo' \$vworld_root_folder/scripts/setup-server.sh"
echo "ssh to your instance"
echo "  ssh root@$public_ip"
echo "connect a client to your instance (macOS)"
echo "  osascript -e 'tell application \"Firefox\" to open location \"file://'\$vworld_root_folder'/vworld-client/index.html?urls=ws://$public_ip:10001\"'"
