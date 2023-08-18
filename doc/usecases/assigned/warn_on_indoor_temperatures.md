# Use Case: Warn/Alarm on in-door Temperatures

## Summary

- **Scope:** Environment Monitoring
- **Level:** User Goal
- **Actors** Opensesame, Environment-Sensor, Nextcloud chat
- **Brief:** Opensesame collects environmental data from the environment sensor and triggers a warning or alarm when a specified temperature threshold is exceeded.
- **Assignee:** Felix
- **Status:** Assigned

## Scenarios

- **Precondition:** 
	- The user has successfully connected the environmental sensor.
	- The warning threshold is set to 28 °C.
	- The alarm threshold is set to 50 °C.
	- The cancel threshold is set to 18 °C
- **Main Success Scenario:** 
	- Opensesame gathers temperature data from the environment sensor.
	- When the warning threshold is above or equal 28°C, Opensesame sends a warning to Nextcloud chat.
	- When the alarm threshold is above or equal 40°C, Opensesame sends an alarm to Nextcloud chat.
	- When the cancel threshold is below 25°C, Opensesame removes the warning/alarm chat.
- **Error scenario:**
	- Unable to read temperature data from the environment sensor; retries reading data; sends an error message to the Nextcloud chat.
- **Postcondition:**
	- Status normal again.
- **Further Requirements:**
	- Information about the temperatures are available in the module.
