# Emulating the Olimex Board

This guide shows how to emulate the Olimex Linux image on your Intel based PC.
It is based [on this forum entry](https://www.olimex.com/forum/index.php?topic=2239.0).

Basically, we use the `qemu-user-static` emulator to emulate ARM binaries on our native kernel, and use `systemd-nspawn` to execute the image like a container.

## Prerequisites

On Debian systems, install the following tools:

```
apt install qemu-user-static systemd-container 
```

On Fedora, use these:

```
dnf install qemu-user-static systemd-container
```

## Running the system

You need to have the [A20 Debian image](http://images.olimex.com/release/a20/) extracted somewhere in your filesystem.
This means you either extracted it from the image file, you mounted the image file, or you have already written it onto a microSD card and mounted that on your system.
As an example, we have extracted the whole image into `/tmp/olimex`.

All that's left is to execute the following command as root:

```
systemd-nspawn --bind=/dev/ttyACM0 -b -D /tmp/olimex
```

Explanations of the parameters:
- `--bind=/dev/ttyACM0` exposes your host system serial device `/dev/ttyACM0` into the chrooted A20 environment.
  This serial device is used in opensesame to read the sensor data.
  
- `-b` advices systemd to act like it's booting the image, so it will run the `init` exectuable of the chrooted environment.

- `-D` specifies the root directory of the A20 image.
  Alternatively, you can use `-i` to specify the image directly, so you don't need to extract the contents of it.

Use the credentials of the image (default root/olimex) to login and work on the system.

If you need network connectivity, you need to configure DNS lookup with the following command:

```
echo 'nameserver 8.8.8.8' >  /run/resolvconf/resolv.conf
```

You need to do this every time you start the container, as anything under `/run` does not get persisted.

If you want to connect into this container via SSH, you need to modify the port in `/etc/sshd/sshd_config` to another port, as port 22 is most likely already used by your host system.

