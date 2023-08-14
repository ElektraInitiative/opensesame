# Use Case: Public Publishing of data for Anyone Interested

## Summary

- **Scope:** Clima-Sensor-US
- **Level:** User Goal
- **Actors** Thies-Clima-Sensor, Opensesame, Opensensemap
- **Brief:** The sensor informations are collected periodically and are published to Opensensemap
- **Assignee:** Felix
- **Status:** Assigned

## Scenarios

- **Precondition:** 
	- The user has configured Opensesame to use Clima-Sensor-US  and to publish to Opensensemap.
	- The web interface must be properly configured, and the corresponding IDs need to be entered in Elektra and in the constants.
- **Main Success Scenario:** 
	- Opensesame retrieves the register values of the Thies-Clima-Sensor.
	- Opensesame generates a JSON format using sensor IDs and register values.
	- Opensesame transmits the JSON string to the Opensensemap API every minute.
- **Error scenario:**
	- Unable to read register value; return the error to Nextcloud
	- Connection to Opensensemap API fails; retry connection to Opensensemap at least once; return error to Nextcloud
- **Postcondition:**
	- Opensensemap has full history of all sensor data.
