# TODO

This file contains what is to be done in August.
Before working on something, issues should be created to clarify details.

The file is ordered by priority.
After something is done, remove it from here.

## Weather station

Assignee: Felix

implement all error codes & send i18n errors into Nextcloud ping chat

publish to https://www.opensensemap.org


## Nextcloud Talk

Assignee: Jannis

create use case: allow to talk to opensesame via Nextcloud

open door

add/rem PIN code

status

lights on/off


## Env

Assignee: Felix

create use case: warn/alarm on in-door temperatures

Implement "Alternative" Env HW with:

MOD-BME280 and MOD-IR-TEMP
https://www.olimex.com/Products/Modules/Sensors/MOD-BME280/open-source-hardware
https://www.olimex.com/Products/Modules/Sensors/MOD-IR-TEMP/open-source-hardware

Configurable corrections on Temperature

warn on fast raising values, next to MQ135 (temperature, CO2, VOC, Pressure)

temperature alarm e.g. 50° (check max temp from sensor) and warning  e.g. 28° (window left open)


## Battery

Assignee: Felix

use case: "on battery" and "low battery" warning via Nextcloud Chat

proper Bat impl https://github.com/svartalf/rust-battery/issues/96


## Nextcloud Analytics

Assignee: Markus

publishing data to [Nextcloud analytics](https://github.com/Rello/analytics/wiki/API#data-add)


## Code Smell

Assignee: Jannis

replace thread spawning and sighandling with async calls

optional/to be discussed: make properly distributed, replace ssh code

ping should be always on the same time

also end alarm mode (instead of ogg123 invocation)

better error handling


## PV

Assignee: Markus

implement all error codes

dump all registers

warn if battery `< 30%`

switching off heat pump if no sunshine (MOD-2 relay)


## Robustness

Assignee: Felix

Writing to SD-Card should be reduced as much as possible, read-only as much as possible

close garage door on reboots (needs up to 2x 10 sec wait)

Buttons and Environment without hardware (i.e. mock): https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs


## Olimex

Assignee: Felix

Bring basic functionality running on hardware

Write release/blog post about outcome of bachelor thesis


## Elektrification Improve Spec

Assignee: Florian

cmd-line arguments, env variables in `files/opensesame.spec`

make nice TOML sections in config files (set via spec)



## Improve Elektra Binding

Assignee: Jannis

allow async

allow generic serialization with specializations (e.g. for bool) maybe using serde https://serde.rs/data-format.html?) (get rid of get_bool and get_hash_map_vec_u8)

important tasks of https://github.com/ElektraInitiative/libelektra/issues/4411 (to be prioritized)

(optional, effort to be discussed:
	better support for arrays in TOML https://github.com/ElektraInitiative/libelektra/issues/4988
	fix hanging bug: https://github.com/ElektraInitiative/libelektra/issues/4981
	)


## Ansible

Assignee: Max/Jannis

put ~/.config into git

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
