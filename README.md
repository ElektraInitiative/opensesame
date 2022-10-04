# Introduction and Goals

Opensesame, or German: "Sesam, öffne dich"
is an awesome home automation software with
focus on door/garage entry.

What already works:

- [X] opening (garage) doors
- [X] switching on entry lights
- [X] doorbell
- [X] fire alert
- [X] HW-reset with PWR-Switch
- [X] events are reported to Nextcloud chats in English or German

For what is missing, see [TODO.md](TODO.md).

Currently there is no full beginner-friendly documentation
on how to setup the system. If you are interested, please
open an issue.

I cannot guarantee that anything will work for you,
I only know it works for me.

## Quality Goals

1. Robust & Secure:
   The automation should be exactly as
   intended, without bypasses or errors.
   A single failure should not affect
   other parts of the system.
   Also on a blackout most systems
   should continue working.

2. Simplicity:
   Flexibility should be only added if
   actually needed and even then in a
   rather static way, e.g. reboots after
   configuration changes are not a
   problem.

# Solution

It is decentralized and runs on several
Olimex A20-OLinuXino-LIME2-e16Gs16M
with Debian Stable.

Kernel: Linux schreibtisch 5.10.105-olimex #090538 SMP Wed Apr 13 09:06:56 UTC 2022 armv7l GNU/Linux

On each Olimex, the same software runs
but differently configured using
Elektra.

E.g. to set a new PIN 1234 for "us":

kdb set user:/sw/libelektra/opensesame/#0/current/validator/us "[14, 15, 13, 15, 11, 15, 7, 15]"

Deployment is done via Ansible, see [an example](/ansible/playbook.yaml).

See [spec](files/opensesame.spec) for available configuration options.

To keep it more simple, the same node
cannot be responsible for having
the same functionality multiple times.

As implementation language Rust is used.


## Hardware

No guarantee for completeness.
Better you open an issue before you actually order something.

As base to actually run opensesame you need:

- https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/

Probably you also want for every device:

- https://www.olimex.com/Products/Modules/Sensors/MOD-ENV/
- https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/
- https://www.olimex.com/Products/OLinuXino/BOXES/BOX-LIME/

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

Furthermore you might need to switch 230V circuits:

- https://en.wikipedia.org/wiki/Snubber
- https://www.homemade-circuits.com/prevent-relay-arcing-using-rc-snubber-circuits/
- This is German but contains many interesting links on how to choose the values of R and C:
  http://www.dse-faq.elektronik-kompendium.de/dse-faq.htm#F.25.1

https://www.olimex.com/wiki/A20-OLinuXino-LIME2#GPIO_description

