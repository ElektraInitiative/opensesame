# Development Setup 
For the development setup you have to do some steps manually, so this page should help you to get your development environment ready to use.

## 1. Clone the project on you local development environment

## 2. Try to build the project (cargo build)
One dependency is libelektra which can be installed by following [these steps](https://github.com/ElektraInitiative/libelektra/blob/master/doc/INSTALL.md)

## 3. Setup specs and states with Elektra
In this step you have to run the `postinst` script, which is located in the debian folder. This script setup the specs, states and mount them into elektra. 

If you want to clear or remove your elektra config you can execute the `postrm` script in the debian folder. 

## 4. After the build and elektra setup was successfully you have to configure your nextcloud with `kdb set` 
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

1. If you get a panic, with no further information, while running opensesame you have to comment the line 164 out (`panic::set_hook` in main). That's a temporary workarount which is already issued [#6](https://github.com/ElektraInitiative/opensesame/issues/6)

2. If you get an error with "TranslationNotFound("de")", then you have to copy the opensesame.mo into your locale/de directory.

```sh
cp files/opensesame.mo /usr/share/locale/de/LC_MESSAGES/
```
or
```sh
cp files/opensesame.mo /usr/share/locale/en/LC_MESSAGES/
```

3. If you get an IO error in the button.rs file, then you have to [mock the I2CDevice](https://docs.rs/i2cdev/0.5.0/i2cdev/mock/struct.MockI2CDevice.html)

4. If you get an error with `thread 'main' panicked at 'Set config failed: Sorry, module  issued error :
: ', src/config.rs:29:17` and the backtrace:
```bash
stack backtrace:
   0: rust_begin_unwind
             at /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/std/src/panicking.rs:578:5
   1: core::panicking::panic_fmt
             at /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/panicking.rs:67:14
   2: opensesame::config::Config::sync
             at ./src/config.rs:29:5
   3: opensesame::config::Config::new
             at ./src/config.rs:20:3
   4: opensesame::main
             at ./src/main.rs:141:27
   5: core::ops::function::FnOnce::call_once
             at /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ops/function.rs:250:5
```

The problem can be solved by executing `/var/lib/dpkg/info/opensesame.postinst`.
