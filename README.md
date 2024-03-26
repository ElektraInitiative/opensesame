[![Build badge](https://build.libelektra.org/job/opensesame/job/master/badge/icon)](https://build.libelektra.org/job/opensesame/job/master/lastBuild/)

# Introduction and Goals

Opensesame, or German: "Sesam, öffne dich" is an awesome home automation software with focus on door/garage entry.

![PIN entry via buttons](/files/pin.jpg)

What already works:

- [x] opening (garage) doors via a novel PIN entry method: you can press and release buttons in any sequence (e.g. press 1, press 2, release 1, release 2)
- [X] switching on entry lights
- [X] doorbell
- [X] fire alert
- [X] HW-reset with PWR-Switch
- [X] events are reported to Nextcloud chats in English or German

For what is missing, see [TODO.md](TODO.md).

## Disclaimer

Currently there is no complete beginner-friendly documentation on how to setup the system.
If you are interested, please open an issue.
[Installation](/doc/Install.md) is not difficult, though.

As hardware hacking is involved I cannot guarantee that anything will work for you.
It works for me and worked for someone else, too.

## Quality Goals

1. Robust & Secure:
   The automation should be exactly as intended.
   A single failure should not affect other parts of the system.
   Also on a blackout battery-powered systems should continue working.

2. Simplicity:
   Flexibility should be only added if actually needed and even then in a rather static way.

# Solution

It is decentralized and runs on several Olimex A20-OLinuXino-LIME2-e16Gs16M with Debian Stable.

Kernel: Linux schreibtisch 5.10.105-olimex #090538 SMP Wed Apr 13 09:06:56 UTC 2022 armv7l GNU/Linux

On each Olimex, the same software runs but differently configured using Elektra.

E.g. to set a new PIN 1234 for "us":

`kdb set user:/sw/libelektra/opensesame/#0/current/validator/us "[14, 15, 13, 15, 11, 15, 7, 15]"`

Deployment is done via Ansible, see [an example](/ansible/playbook.yaml).

See [spec](/files/opensesame.spec) for available configuration options.

To keep it more simple, the same node cannot be responsible for having the same functionality multiple times.

As implementation language Rust is used.


## Hardware

No guarantee for completeness.
Better you open an issue before you actually order something.

As base to actually run opensesame you need:

- https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/

If you want to try it out on your normal PC, we have provided an [emulation guide](doc/Emulating-Olimex.md).

Probably you also want for every device:

- https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/
- https://www.olimex.com/Products/OLinuXino/BOXES/BOX-LIME/
- https://www.olimex.com/Products/Modules/Sensors/MOD-ENV/ (unfortunately not available anymore, we investigate alternatives)

To open the doors you need:

- 6x https://www.olimex.com/Products/Components/Switches/WATER-PROOF-BUTTONS/
- 2x https://www.olimex.com/Products/Modules/IO/MOD-IO2/
- e.g. https://www.olimex.com/Products/Components/Misc/DOOR-LOCK/

For PWR Switch (switch off mod-io, sensors etc.) you need:

- https://www.olimex.com/Products/Duino/Shields/PWR-SWITCH/

For smoke detection you need:

- [Arduino](arduino) or compatible Olimex products
- up to 12x https://www.olimex.com/Products/Components/Sensors/Gas/SNS-MQ135/

To detect if Garage door is at end position:

- e.g. Basic Switch Hinge Lever Low OP

Furthermore to switch 230V circuits (with the relays of the MOD-IO2) you might need:

- https://en.wikipedia.org/wiki/Snubber
- https://www.homemade-circuits.com/prevent-relay-arcing-using-rc-snubber-circuits/
- This is German but contains many interesting links on how to choose the values of R and C:
  http://www.dse-faq.elektronik-kompendium.de/dse-faq.htm#F.25.1

