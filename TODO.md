## Smaller Features

Configurable corrections on Temperature

warn on other fast raising values, next to MQ135 (temperature, CO2, VOC, Pressure)

temperature alarm e.g. 70° (check max temp from sensor) and warning 28° (window left open)


## Bigger Features

allow to also talk to sensors via Nextcloud, e.g.:
- open door
- set PIN code

publishing data to [Nextcloud analytics](https://github.com/Rello/analytics/wiki/API#data-add)

reading data from photovoltaik

switching off heat pump if no sunshine e.g. using https://github.com/chrishrb/hoval-gateway/issues/7


## Usability

more interesting info in ping (give various modules a chance, averages of errors happened, humidity?)


## Code Smell

replace thread spawning and sighandling with async calls

make properly distributed , replace ssh code

ping should based on time

also end alarm mode (instead of ogg123 invocation)

better error handling

avoid double loop in main.rs

env1/env2 allow to always use two sensors, or only one sensor fixed


## Completeness

CCS811 Sensor
- implement hysteresis
- implement error codes
- implement compensations
- restore baseline
- add unit tests

fix sensor test cases



## Robustness

allow to power-off mod-IOs via PWR-SWITCH

make some not so loud sound on bootup+visual indication at buttons (to hear reboot loops)

Test if it survives power-off/on

Writing to SD-Card should be reduced as much as possible, read-only as much as possible



## Proper start/shutdown

close garage door (needs up to 2x 10 sec wait)


## Elektrification

make nice TOML sections in config files

Umlauts in config

allow generic serialization with specializations (e.g. for bool) using serde https://serde.rs/data-format.html?) (get rid of get_bool and get_hash_map_vec_u8)

internal notification on state changes

fix boolean (0 instead of false, 1 instead of true)

allow reload of everything (reinit everything)

Buttons and Environment without hardware (i.e. mock): https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs

put into own crate


## Rust

cross-compile

use rustfmt

https://github.com/viperproject/prusti-dev

refactor parts to be a lib? https://doc.rust-lang.org/stable/book/ch12-03-improving-error-handling-and-modularity.html

Destructors: https://doc.rust-lang.org/stable/reference/destructors.html

life-times in closures, lifetime within structs, ...

https://marabos.nl/atomics/memory-ordering.html

use more https://github.com/rust-embedded/embedded-hal


## Ansible

put .config into git

default editor not working

also install opensesame (via repo)

use Handlers to send SIGHUP only as needed https://docs.ansible.com/ansible/latest/user_guide/playbooks_handlers.html

/usr/share/alsa/alsa.conf

	"pcm.front cards.pcm.front" must be updated to "pcm.front cards.pcm.default"
	see also https://forums.raspberrypi.com/viewtopic.php?t=136974

/etc/default/keyboard (dpkg-reconfigure keyboard-configuration) not correct

	  # keyboard-configuration  keyboard-configuration/layoutcode       string  at
	  # keyboard-configuration  keyboard-configuration/modelcode        string  pc105
	  # keyboard-configuration  keyboard-configuration/model    select  Generic 105-key PC (intl.

wrong timezone

static IPv4 and IPv6 addresses

add public gpg keys for login
