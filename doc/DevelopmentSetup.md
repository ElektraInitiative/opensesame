# Development Setup 
For the development setup you have to do some steps manually, so this page should help you to get your development environment ready to use.

## 1. Clone the project on you local development environment

## 2. Try to build the project (cargo build)
One dependency is libelektra which can be installed by following [these steps](https://github.com/ElektraInitiative/libelektra/blob/master/doc/INSTALL.md)

## 3. Setup specs and states with Elektra
In this step you have to run `cd debian && postinst`. This script mounts the configuration specification and states into elektra.

If you want to clear or remove your elektra config you can execute the `postrm` script in the debian folder. 

## 4. After the build and elektra setup was successfully you have to configure your nextcloud with `kdb set` 
As minimum you have to add [nextcloud/url], [nextcloud/chat], [nextcloud/chat/licht], [nextcloud/chat/ping], [nextcloud/user] and [nextcloud/pass]. This is done with the following statements:

```sh
kdb set user:/sw/libelektra/opensesame/#0/current/nextcloud/url "https://nextcloud.my-server.com/nextcloud"
kdb set user:/sw/libelektra/opensesame/#0/current/nextcloud/chat "<token>"
kdb set user:/sw/libelektra/opensesame/#0/current/nextcloud/chat/licht "<token>"
kdb set user:/sw/libelektra/opensesame/#0/current/nextcloud/chat/ping "<token>"
kdb set user:/sw/libelektra/opensesame/#0/current/nextcloud/user "<user>"
kdb set user:/sw/libelektra/opensesame/#0/current/nextcloud/pass "<password>"
```

The chat-token can be extracted from the chat-url, e.g. `https://nextcloud.my-server.com/nextcloud/index.php/call/<token>`

## 5. If you have troubles see [this doc](Troubleshooting.md) 


