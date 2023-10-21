# TODO

This file contains ideas to be done.
Before working on something, issues must be created to clarify details.

This file is ordered by priority.
After something is done, remove it from here.

## Nextcloud Talk

Assignee: Jannis

allow to talk to opensesame via Nextcloud

add/rem PIN code

## Env

Configurable corrections on Temperature

warn on fast raising values, next to MQ135 (temperature, CO2, VOC, Pressure)

## Battery

proper Bat impl https://github.com/svartalf/rust-battery/issues/96

## Nextcloud Analytics

publishing data to [Nextcloud analytics](https://github.com/Rello/analytics/wiki/API#data-add)

## Code Smell

better error handling

replace ssh code

## PV

implement all error codes

warn if battery `< 30%`

switching off heat pump if no sunshine (MOD-2 relay)

## Robustness

Writing to SD-Card should be reduced as much as possible, read-only as much as possible

close garage door on reboots (needs up to 2x 10 sec wait)

Buttons and Environment without hardware (i.e. mock): https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs

## Elektrification Improve Spec

cmd-line arguments, env variables in `files/opensesame.spec`

make nice TOML sections in config files (set via spec)

## Improve Elektra Binding

allow async

allow generic serialization with specializations (e.g. for bool) maybe using serde https://serde.rs/data-format.html?) (get rid of get_bool and get_hash_map_vec_u8)

important tasks of https://github.com/ElektraInitiative/libelektra/issues/4411 (to be prioritized)

better support for arrays in TOML https://github.com/ElektraInitiative/libelektra/issues/4988

fix hanging bug: https://github.com/ElektraInitiative/libelektra/issues/4981

## Ansible

also install opensesame (via repo)

/usr/share/alsa/alsa.conf

	"pcm.front cards.pcm.front" must be updated to "pcm.front cards.pcm.default"
	see also https://forums.raspberrypi.com/viewtopic.php?t=136974

