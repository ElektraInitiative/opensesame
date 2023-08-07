# Opensensemap
After you've setup your account and logged in on [opensensemap.org](https://www.opensensemap.org/), you need to do following steps:

## 1. Add a senseBox

### 1.1 Choose a name for your station
### 1.2 Choose exposure (`indoor/outdoor/mobile`)
### 1.3 Set Latitude and Longitude or select location in map
### 1.4 Use option Hardware, with `Manual configuration`
This is needed because we the Thies-Clima-Sensor isn't a default hardware configuration.

## 2. Add Sensors

### 2.1 Choose a icon
### 2.2 Set a name/phenomenon, unit and type 
Assign a name to represent this phenomenon.
The unit of the phenomenon could be `m/s`, `°C` or similar.
Use the variable names from the register address constants in clima_sensor_us.rs to specify the type.

## 3. Extract senseBox-ID, Access-Token and Sensor-IDs

### 3.1 senseBox-ID can be copied on the `Dashboard`
### 3.2 Access-Token can be copied in `Dashboard/EDIT/Security`
### 3.3 Sensor-IDs can be copied in `Dashboard/EDIT/Sensors`