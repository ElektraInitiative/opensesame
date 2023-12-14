# boot with A20-Debian-Server-SD

# latest A20-Debian-Server-SD images at:

http://images.olimex.com/release/a20/

``` bash
$ wget https://images.olimex.com/release/a20/A20-OLinuXino-bullseye-minimal-20230515-130040.img.7z
$ 7z x A20-OLinuXino-bullseye-minimal-20230515-130040.img.7z
Compressed: 179781272
$ md5sum -c A20-OLinuXino-bullseye-minimal-20230515-130040.img.md5
A20-OLinuXino-bullseye-minimal-20230515-130040.img: OK

$ sudo cp A20-OLinuXino-bullseye-minimal-20230515-130040.img /dev/sdh
$ sudo sync
```

# boot into the new system

## ssh open by default
## user/pass: olimex/olimex

## now install on emmc device:
```bash 
sudo olinuxino-sd-to-emmc
```
## or to SSD:
``` bash
sudo olinuxino-sd-to-sata
```

# now restart without ssd
```bash
sudo shutdown -r now
```


## Set passwords
```bash
passwd
sudo passwd
```
# TODO: add public gpg keys for login

## Hostname
```bash
cat $HOSTNAME > /etc/hostname
sudo hostname $HOSTNAME
```

## Static IP

TODO

## i2ctools
```bash
sudo adduser olimex i2c
sudo chown root.i2c /dev/gpiochip0
sudo chmod 660 /dev/gpiochip0
```
## allow watchdog triggering TODO, doesn't allow restarts
```bash
sudo chgrp plugdev /dev/watchdog
sudo chmod 660 /dev/watchdog
```

## Elektra

https://www.libelektra.org/docgettingstarted/installation
```bash
sudo mv /usr/lib/python3.9/site-packages /usr/lib/python3.9/dist-packages
```

## Basesystem setup
```bash
dpkg-reconfigure locales
```

# etc...


## Ansible Opensesame setup

change ansible/playbook.yaml and then:
```bash
ansible/run.sh
```



## For elektra-sys in Cargo
```bash
sudo apt-get install llvm-dev libclang-dev # needed for cargo when Elektra is installed
export LIBCLANG_PATH=/usr/lib/llvm-11/lib
```
## For gettext in Cargo
```bash
sudo apt install gettext
export GETTEXT_SYSTEM=1
```




## opensesame
```bash
git clone https://github.com/ElektraInitiative/opensesame.git

cd opensesame

sudo apt install librust-openssl-dev librust-libz-sys-dev librust-libssh2-sys-dev librust-object-dev librust-tokio+default-dev librust-hashbrown-dev

cargo install cargo-deb

cargo deb # build package

dpkg -i ...
```

## Backup

After everything was done successfully, you probably want to backup:
```bash
/usr/bin/rsync -ax -HS --delete --backup --backup-dir=/home/data/Backup/olimex-delete root@192.168.178.55:/ /home/data/Backup/olimex
```
