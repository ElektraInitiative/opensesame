## Weather station

libmodbus-rs fix

implement all error codes

dump all registers

note when raising `> 23°` for closing
warn when raising `> 30°` and no wind
warn when raising `> 35°`
(and remove warning again if `< 20°`)

publish to https://www.opensensemap.org


## Nextcloud Talk

(allow to also talk to opensesame via Nextcloud)

open door

add/rem PIN code

status

lights on/off


## Env

Implement "Alternative" Env HW with:

MOD-BME280 and MOD-IR-TEMP
https://www.olimex.com/Products/Modules/Sensors/MOD-BME280/open-source-hardware
https://www.olimex.com/Products/Modules/Sensors/MOD-IR-TEMP/open-source-hardware

Configurable corrections on Temperature

warn on fast raising values, next to MQ135 (temperature, CO2, VOC, Pressure)

temperature alarm e.g. 50° (check max temp from sensor) and warning  e.g. 28° (window left open)


## Battery

proper Bat impl https://github.com/svartalf/rust-battery/issues/96

"on battery" and "low battery" warning (freezer would be without power then)


## Nextcloud Analytics

publishing data to [Nextcloud analytics](https://github.com/Rello/analytics/wiki/API#data-add)


## Code Smell

replace thread spawning and sighandling with async calls

make properly distributed , replace ssh code

ping should based on time

also end alarm mode (instead of ogg123 invocation)

better error handling

avoid double loop in main.rs

env1/env2 allow to always use two sensors, or only one sensor fixed


## PV

implement all error codes

dump all registers

warn if battery `< 30%`

switching off heat pump if no sunshine (MOD-2 relay)


## Robustness

Writing to SD-Card should be reduced as much as possible, read-only as much as possible

close garage door on reboots (needs up to 2x 10 sec wait)


## Elektrification

cmd-line arguments

make nice TOML sections in config files

Umlauts in config

allow generic serialization with specializations (e.g. for bool) using serde https://serde.rs/data-format.html?) (get rid of get_bool and get_hash_map_vec_u8)

internal notification on state changes

Buttons and Environment without hardware (i.e. mock): https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs


## Rust

Destructors: https://doc.rust-lang.org/stable/reference/destructors.html

life-times in closures, lifetime within structs, ...

use more of https://github.com/rust-embedded/embedded-hal


## Ansible

put .config into git

default editor not working

also install opensesame (via repo)

/usr/share/alsa/alsa.conf

	"pcm.front cards.pcm.front" must be updated to "pcm.front cards.pcm.default"
	see also https://forums.raspberrypi.com/viewtopic.php?t=136974

/etc/default/keyboard (dpkg-reconfigure keyboard-configuration) not correct

	  # keyboard-configuration  keyboard-configuration/layoutcode       string  at
	  # keyboard-configuration  keyboard-configuration/modelcode        string  pc105
	  # keyboard-configuration  keyboard-configuration/model    select  Generic 105-key PC (intl.

add public gpg keys for login
