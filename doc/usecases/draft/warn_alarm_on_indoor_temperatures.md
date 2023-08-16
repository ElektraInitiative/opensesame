# Use Case: Warn/Alarm on in-door Temperatures

## Summary

- **Scope:** Environment Monitoring
- **Level:** User Goal
- **Actors** Opensesame, Environment-Sensor, Nextcloud
- **Brief:** Opensesame collects environmental data from the environment sensor and triggers a warning or alarm when a specified temperature threshold is exceeded.
- **Assignee:** 
- **Status:** Draft

## Scenarios

- **Precondition:** 
	- The user has successfully connected the environmental sensor.
	- The warning threshold is set to 28 °C.
	- The alarm threshold is set to 50 °C.
	- The cancel threshold is set to 18 °C
- **Main Success Scenario:** 
	- Opensesame gathers temperature data from the environment sensor.
	- If the warning threshold is exceeded, Opensesame sends a warning to Nextcloud.
	- If the alarm threshold is exceeded, Opensesame sends an alarm to Nextcloud.
	- If the cancel threshold is undershot, Opensesame removes the warning/alarm.
	- Implementing hysteresis prevents rapid switching between warning and alarm states when the temperature is between 28°C and 50°C.
- **Error scenario:**
	- Unable to establish a connection to Nextcloud; 
	- Unable to read temperature data from the environment sensor; retries reading data; sends an error message to Nextcloud
- **Postcondition:**
	- History of alarms and warnings stored in Nextcloud.
