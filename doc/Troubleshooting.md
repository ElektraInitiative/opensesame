# Troubleshooting

## DevelopmentSetup 

### 1. If you get a panic, with no further information, while running opensesame you have to comment the line 164 out (`panic::set_hook` in main). That's a temporary workarount which is already issued [#6](https://github.com/ElektraInitiative/opensesame/issues/6)

### 2. If you get an error with "TranslationNotFound("de")", then you have to copy the opensesame.mo into your locale/de directory.

```sh
cp files/opensesame.mo /usr/share/locale/de/LC_MESSAGES/
```
or
```sh
cp files/opensesame.mo /usr/share/locale/en/LC_MESSAGES/
```

### 3. If you get an IO error in the button.rs file, then you have to [mock the I2CDevice](https://docs.rs/i2cdev/0.5.0/i2cdev/mock/struct.MockI2CDevice.html)

### 4. If you get an error with `thread 'main' panicked at 'Set config failed: Sorry, module  issued error :: ', src/config.rs:29:17` and the backtrace:

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

### 5. If you get an error with `No metadata "mountpoint" found on key ...` by executing `./debian/postinst`
You need to execute the `postinst` file directly in the `debian` directory.
```
cd debian/ && ./postinst
```