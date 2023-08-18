# Use Case: 'on Battery' and 'low Battery' warning via Nextcloud Chat

## Summary

- **Scope:** Battery Monitoring
- **Level:** User Goal
- **Actors** Opensesame, Nextcloud
- **Brief:** Opensesame monitors the battery status and sends corresponding alerts to the Nextcloud chat.
- **Assignee:** 
- **Status:** Draft

## Scenarios

- **Precondition:** 
	- The user has connected a battery.
	- The user has configured a 'low battery' threshold.
- **Main Success Scenario:** 
	- Opensesame collects battery information, including the battery percentage and power usage status.
	- If the battery is not in use and the 'low battery' threshold is not exceeded, no action is taken.
	- If the battery is in use, a warning is sent via the Nextcloud chat.
	- If the battery level falls below the 'low battery' threshold, a warning is sent via the Nextcloud chat. 
- **Error scenario:**
	- Battery could not be detected;
- **Postcondition:**
	- Information about the battery state is available, and power failure detection is enabled.
