# Development Setup 
For the devolpomnet setup you have to do some steps manually, so this page should help you to get you development environment (devl-env) ready to start.

## 1. Clone the project on you local devl-env

## 2. Try to build the project (cargo build)
One importent dependencies is libelektra which can be installed by following [these steps](https://github.com/ElektraInitiative/libelektra/blob/master/doc/INSTALL.md)

## 3. After the build was successfully you have to configure your nextcloud in the files/opensesame.spec file
Therefor you have to edit the [nextcloud/url], [nextcloud/chat], [nextcloud/chat/licht], [nextcloud/chat/ping] and add [nextcloud/user], [nextcloud/pass]. Instead of the 'required ='-option you have to write 'default ='.
```sh
[nextcloud/url]
description = URL to be used for sending messages.
default = https://nextcloud.my-server.com/
```

```sh
[nextcloud/chat]
description = which chat to use for sending messages.
default = <Token>
check/length/max = 8
```

```sh
[nextcloud/user]
description = which nextcloud user
default = <username>
```

```sh
[nextcloud/pass]
description = password of the user
default = <password>
```

The chat-token can be extracted from the chat-url, e.g. https://nextcloud.my-server.com/nextcloud/index.php/call/<token>

# Troubleshoot 

1. If you get a panic,with no further information, while running opensesame you have to comment the line 164 out (in main). That's a temporary workarount which is already issued [#6](https://github.com/ElektraInitiative/opensesame/issues/6)

```rust
panic::set_hook(Box::new(|panic_info| {
		let (filename, line) = panic_info
			.location()
			.map(|loc| (loc.file(), loc.line()))
			.unwrap_or(("<unknown>", 0));
		let cause = panic_info
			.payload()
			.downcast_ref::<String>()
			.map(String::deref);
		let cause = cause.unwrap_or_else(|| {
			panic_info
				.payload()
				.downcast_ref::<&str>()
				.map(|s| *s)
				.unwrap_or("<cause unknown>")
		});
		let mut config: Config = Config::new(CONFIG_PARENT);
		let nc: Nextcloud = Nextcloud::new(&mut config);
		let text = gettext!("A panic occurred at {}:{}: {}", filename, line, cause);
		nc.ping(text.clone());
		eprintln!("{}", text);
	}));
``` 

2. If you get an error with "TranslationNotFound("de")", then you have to copy the opensesame.so into your local/de directory

```sh
cp file/opensesame.so /usr/share/local/de/LC_MESSAGES/
```

3. If you get an IO error in the button.rs file, then you have to [mock the I2CDevice](https://docs.rs/i2cdev/0.5.0/i2cdev/mock/struct.MockI2CDevice.html)
