# Development Setup 
For the devolpomnet setup you have to do some steps manually, so this page should help you to get you development environment (devl-env) ready to start.

## 1. Clone the project on you local devl-env

## 2. Try to build the project (cargo build)
One importent dependencies is libelektra which can be installed by following [these steps](https://github.com/ElektraInitiative/libelektra/blob/master/doc/INSTALL.md)

## 3. After the build was successfully you have to configure your nextcloud with `kdb set` 
Therefore you have to add [nextcloud/url], [nextcloud/chat], [nextcloud/chat/licht], [nextcloud/chat/ping], [nextcloud/user] and [nextcloud/pass]. This is done with the following statements:

```sh
kdb set system:/sw/libelektra/opensesame/#0/current/nextcloud/url "https://nextcloud.my-server.com/nextcloud"
kdb set system:/sw/libelektra/opensesame/#0/current/nextcloud/chat "<token>"
kdb set system:/sw/libelektra/opensesame/#0/current/nextcloud/chat/licht "<token>"
kdb set system:/sw/libelektra/opensesame/#0/current/nextcloud/chat/ping "<token>"
kdb set system:/sw/libelektra/opensesame/#0/current/nextcloud/user "<user>"
kdb set system:/sw/libelektra/opensesame/#0/current/nextcloud/pass "<password>"
```

The chat-token can be extracted from the chat-url, e.g. `https://nextcloud.my-server.com/nextcloud/index.php/call/<token>`

# Troubleshoot 

1. If you get a panic,with no further information, while running opensesame you have to comment the line 164 out (`panic::set_hook` in main). That's a temporary workarount which is already issued [#6](https://github.com/ElektraInitiative/opensesame/issues/6)

2. If you get an error with "TranslationNotFound("de")", then you have to copy the opensesame.mo into your local/de directory

```sh
cp file/opensesame.mo /usr/share/local/de/LC_MESSAGES/
```

3. If you get an IO error in the button.rs file, then you have to [mock the I2CDevice](https://docs.rs/i2cdev/0.5.0/i2cdev/mock/struct.MockI2CDevice.html)
