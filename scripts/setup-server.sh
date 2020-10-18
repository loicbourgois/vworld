cp $vworld_root_folder/ansible/inventory.yml $vworld_root_folder/ansible/inventory.yml.save
sed -i . "s/MY_HOST/$vworld_host/" "$vworld_root_folder/ansible/inventory.yml"
ansible-playbook \
  --limit all \
  --inventory $vworld_root_folder/ansible/inventory.yml \
  --extra-vars "ansible_user=root" \
  --extra-vars "git_branch=$git_branch" \
  --extra-vars "vworld_config=$vworld_config" \
  $vworld_root_folder/ansible/setup.yml
mv $vworld_root_folder/ansible/inventory.yml.save $vworld_root_folder/ansible/inventory.yml
