# Low Battery Warning

## Summary

- **Scope:** Battery Monitoring
- **Level:** User Goal
- **Actors** System, Nextcloud chat
- **Brief:** Opensesame monitors the battery status and sends corresponding alerts to the Nextcloud chat.

## Scenarios

- **Precondition:** 
	- The user has connected a battery.
	- The device is on grid.
- **Main Success Scenario:** 
	- When System is disconnected from grid for a minute, when the battery is below 80%, a warning is sent via the Nextcloud chat.
	- When the battery level of the system falls below the 30% threshold, a further 'low battery' warning is sent via the Nextcloud chat.
	- When the system is again on grid, the warning(s) are canceled.
- **Error scenario:**
	- Battery could not be detected.
- **Postcondition:**
	- Status normal again.
- **Further Requirements:**
	- Information about the battery state is available in the module, and power failure detection is enabled.
