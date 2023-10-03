# OpenSesame

## audio.rs
This module handles audio output for playing fire alarms and bell sounds. This module can receive commands from the Module Buttons (Bell), Environment (FireAlarm), Nextcloud (FireAlarm, Bell), and Signals (FireAlarm, Bell).

## bat.rs
Checks the battery capacity every ten minutes and outputs to Nextcloud if it falls below 50%. If it goes below 50%, the threshold for the next Nextcloud message is set to 40%.

## buttons.rs
This module implements the `do_reset` if an error occurs on the MOD-IO2 modules. In the `async get_background_task`, we implemented how the buttons were handled in the old main, including the validator and command receiver. The command receiver receives commands like `opendoor` (used by Garage, Nextcloud), `ring_bell` (used by Environment, Signals), `switchlights` (used by Garage, Nextcloud), and `RingBellAlarm` (used by Signals).

## clima_sensor_us.rs 
This module works independently, sending warnings to Nextcloud and publishing to opensensemap. We needed to implement `Send` to use libmodbus with async functions.

## config.rs
No changes were made to this module.

## environment.rs
Changed the functions `rememberBaseline`, `restoreBaseline`, and added a `Muext` of state. We moved `handle_environment` from the main into the `get_background_task`. This module receives commands (RestoreBaseline, RememberBaseline) from Signals.

## garage.rs
This module is checked at intervals of 10 milliseconds, triggering Nextcloud Chat or Buttons commands if the button is pressed. Future changes will involve removing the interval and implementing trigger-oriented events because GPIO pins from the Olimex board are used, along with interrupts.

## mod_ir_temp.rs
This module is triggered at given intervals and warns by sending Nextcloud messages.

## nextcloud.rs 
Implements two loops: one for sending (`message_sender_loop`) messages and status to Nextcloud, and the other for receiving (`command_loop`) messages/commands from Nextcloud. Commands can be sent via Nextcloud chat by typing "\opensesame" to open the door, or other commands like "\ring_bell", "\fire_alarm", "\status", and "\switchlights true true".

## ping.rs
This module sends a ping message to Nextcloud if it receives the `SendPing` event. Other functions update `Env`, `EnvStatus`, `EnvError`, and `BatCapacity`, but these commands aren't used yet. They need to be triggered at the right spot in Environment and Battery to keep the ping up to date.

## pwr.rs 
No significant changes were made to this module.

## sensors.rs 
No major changes, only the `get_background_task` was added with a similar implementation as in the old version.

## signals.rs 
This module listens to system signals and executes the same events as in the old version.

## ssh.rs 
Changed to an async function.

## types.rs
This module contains the error types defined so far.

## validator.rs
No changes were made to this module.

## watchdog.rs
This background task is triggered every few seconds and writes to the specified file. The `test_get_background_task` function is used to simulate the watchdog stopping writing to the watchdog file.

## main.rs
In this loop, we initialize every module if it is enabled. Additionally, the MPSC (Multiple Producer, Single Consumer) channels are initialized here. To configure each module without encountering issues related to multiple access to one config object, we read the configuration in the main.

