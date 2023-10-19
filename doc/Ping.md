# Ping Module
This module sends a ping message to Nextcloud if it receives the `SendPing` event.
Other functions update `Env`, `EnvStatus`, `EnvError`, and `BatCapacity`, but these commands aren't used yet.
They need to be triggered at the right spot in Environment and Battery to keep the ping up to date.