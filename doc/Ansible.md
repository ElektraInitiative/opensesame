# Ansible

## Setting up Development-Board

### Create image with static IP and password for olimex
To begin, mount the default image and run it using `nspawn-systemd`,  as described [hier](Emulating-Olimex.md). 
Next, configure the static IP address to match your network setup by editing the file `/etc/network/interfaces` with the following details:
``` bash
auto eth0
iface eth0 inet static 
  address 192.168.0.10
  netmask 255.255.255.0
  gateway 192.168.0.1
``` 
You can also change the password for the user `olimex`` with the following command:
``` bash
passwd olimex
```
After completing the above steps, copy the image to an SD-Card. If the image is already on the SD-Card, proceed to boot the Olimex board using this SD-Card.
### Execute Ansible Playbook
Finally, all that's left to do is run the `run.sh`  script found in the `ansible/dev-setup` directory.