# Use Case: public publishing of data for anyone interested

## Summary

- **Scope:** Clima-Sensor-US - Module
- **Level:** Backend
- **Actors** Thies-Clima-Sensor, Opensensemap
- **Brief:** The sensor informations are collected periodically and are published to opensensemap
- **Assignee:** Felix
- **Status:** Assigned

## Scenarios

- **Precondition:** 
	- The user has configured opensesame to use sensor mode.
	- Sensor IDs are set in constants.
	- Update frequency is set.
- **Main Success Scenario:** 
	- Retrieve the register values of the clima sensor.
	- Generate a JSON format using sensor IDs and register values.
	- Transmit the JSON string to the Opensensemap API.
- **Error scenario:**
	- Unable to read register value; return an error to the calling method.
	- Connection to Opensensemap API fails; return an error to the calling method.
- **Postcondition:**
	- The method for publishing to Opensensemap is triggered periodically.
