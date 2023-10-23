# Nextcloud Talk

## Summary

- **Scope:** Nextcloud Talk
- **Level:** User Goal
- **Actors** User, Chat (Nextcloud Talk), Opensesame (the agents)
- **Brief:** Allow chatting with Opensesame via Nextcloud talk
- **Assignee:** Jannis

## Scenarios

- **Precondition:** 
  - The device is on grid.
- **Main Success Scenario:** 
  - The user writes `?` and gets a list of available commands.
  - The user writes `open!` (or `öffnen!`) and the door opens.
  - The user writes `open?` (or `offen?`) and the open/close status gets reported (garage only).
  - The user writes `lights [in/out]!` (or `Licht [innen/aussen]!`) and the indoor/outdoor (or both default) lights get switched on or off.
  - The user writes `lights?` (or `Licht?`) and gets a report of which lights are on or off.
  - The user writes `battery?` (or `Batterie?`) and the battery status gets reported.
  - The user writes `weather?` (or `Wetter?`) and the weather data gets reported.
  - The user writes `indoor climate?` (or `Innenklima?`) and the indoor environment data gets reported.
  - The user writes `status?` (or `Status?`) and opensesame reports the user's nextcloud status into the chat.
  - The user writes `sensors?` (or `Sensoren?`) and the current sensor data gets reported.
  - The user writes `alarm!` (or `Alarm!`) and a bell alarm gets triggered.
  - The user writes `all clear!` (or `entwarnen!`) and all states (alarms, audio, bell, etc.) get canceled.
  - The user writes `bell!` (or `Glocke!`) and a bell is ringed.
  - The user writes `quit!` (or `beenden!`) and opensesame quits (and automatically restarts).
  - The user writes `time?` (or `Uhrzeit?`) and opensesame report their current time.
  - The user writes `play [audio file]!` (or `abspielen [audio file]!`) and an audio file is played (default bell).
  - The user writes `code?` and gets a list of names and codes.
  - The user writes `code add <name> <pin>!`, `code del <name>!` or `code set <name> <pin>!` (or `pin hinzufügen/löschen/ändern!`) in the chat and new validator codes are added or removed.
- **Alternate Scenario:** 
  - The user pre- or postfixes one of the commands with `@user` and only the specific opensesame instance with `user` as Nextcloud user responds to the question.
    Example: `@garage öffnen!`
- **Error scenario:**
  - The communication with Nextcloud doesn't work.
- **Postcondition:**
  - Bidirectional connection with Nextcloud talk chat.
- **Further Requirements:**
  - Module is extensible such that new chat commands can be introduced later.
  - Commands are case-insensitive.
  - Shortcuts with only writing the first letters of commands should work, too.
