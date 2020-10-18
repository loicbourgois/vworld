cp $vworld_root_folder/ansible/inventory.yml $vworld_root_folder/ansible/inventory.yml.save
sed -i . "s/MY_HOST/$vworld_host/" "$vworld_root_folder/ansible/inventory.yml"
ansible-playbook \
  --limit all \
  --inventory $vworld_root_folder/ansible/inventory.yml \
  --extra-vars "ansible_user=root" \
  --extra-vars "git_branch=$git_branch" \
  --extra-vars "configuration_name=$configuration_name" \
  $vworld_root_folder/ansible/setup.yml
mv $vworld_root_folder/ansible/inventory.yml.save $vworld_root_folder/ansible/inventory.yml
echo "Run vworld through docker"
echo "  ssh root@$vworld_host"
echo "  export vworld_root_folder='/root/vworld'"
echo "  configuration_name='demo' \$vworld_root_folder/scripts/vworld-server-start.sh"
