# Public Publishing of weather data

## Summary

- **Scope:** Clima-Sensor-US Monitoring
- **Level:** User Goal
- **Actors** Thies-Clima-Sensor, Opensesame, Opensensemap, Nextcloud chat
- **Brief:** The sensor informations are collected periodically and are published to Opensensemap
- **Assignee:** Felix
- **Status:** Assigned

## Scenarios

- **Precondition:** 
	- The user has configured Opensesame to use Clima-Sensor-US and to publish to Opensensemap.
	- The [web interface of Opensensemap](www.opensensemap.org) is properly configured, and the corresponding IDs are entered in Elektra.
- **Main Success Scenario:** 
	- Opensesame retrieves the register values of the Thies-Clima-Sensor.
	- Opensesame generates a JSON format using sensor IDs and register values.
	- Opensesame transmits the JSON string to the Opensensemap API every minute.
	- When the outdoor temperature is above or equal 22°C, a warning to close the windows is sent via Nextcloud chat.
	- When the outdoor temperature is above or equal 35°C OR 30°C and no wind, a heat warning is sent via Nextcloud chat.
	- Once the outdoor temperature drops below 20°C, the warnings are canceled in the Nextcloud Chat.
- **Error scenario:**
	- Unable to read register value; return the error to Nextcloud chat.
	- Connection to Opensensemap API fails; retry connection to Opensensemap at least once; return error to the Nextcloud chat.
- **Postcondition:**
	- Opensensemap has full history of all sensor data.
- **Further Requirements:**
	- Information about weather data is available in the module.
