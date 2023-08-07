# Opensensemap
After you've setup your account and logged in on [opensensemap.org](https://www.opensensemap.org/), you need to do following steps:

## 1. Adding a senseBox

### 1.1 Naming Your Station
Choose a distinct name for your senseBox station.
### 1.2 Specifying Exposure
Select the appropriate exposure setting: indoor, outdoor, or mobile.
### 1.3 Geographical Coordinates
Enter the latitude and longitude or pick a location on the map.
### 1.4 Hardware Configuration
Choose the "Manual configuration" option under hardware. 
This is necessary as the Thies-Clima-Sensor isn't a default hardware configuration.

## 2. Adding Sensors

### 2.1 Sensor Icon
Choose an icon to represent the weather phenomenon you're measuring.
### 2.2 Sensor Details 
Provide a name, unit (e.g., m/s, °C), and specify the sensor type using the variable names from the register address constants in clima_sensor_us.rs.

## 3. Obtaining IDs

### 3.1 senseBox-ID
Find the senseBox-ID in the `Dashboard` section.
### 3.2 Access-Token
Locate the Access-Token in `Dashboard/EDIT/Security``.
### 3.3 Sensor-IDs
Retrieve the Sensor-IDs from `Dashboard/EDIT/Sensors`.

## 4. Configuring Opensesame for Weatherstation Usage

### 4.1 Nextcloud Setup 
Configure Nextcloud as outlined in [DevelopmentSetup.md](./DevelopmentSetup.md).
### 4.2 Weather Station Configuration
Use the following commands in your terminal to configure Opensesame for your weather station:
```bash
kdb set user:/sw/libelektra/opensesame/#0/current/weatherstation/enable "1"
kdb set user:/sw/libelektra/opensesame/#0/current/weatherstation/opensensemap/id "<opensensemap-box-id>"
kdb set user:/sw/libelektra/opensesame/#0/current/weatherstation/opensensemap/token "<opensensemap-access-token>"
```
### 4.3 Enabling Sensor Mode
Enable sensor mode by creating a sensor profile with the following settings:
```bash
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/loc "Weatherstation"
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/quality "++"
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/bell 500
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/alarm 600
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/min 0
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/avg 25
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/max 100 
kdb set user:/sw/libelektra/opensesame/#0/current/sensors/#0/pin "Shield-Lime2 uext2"
```

### 4.4 Environment Name
Set an environment name using the command:
```bash
kdb set user:/sw/libelektra/opensesame/#0/current/environment/name "<any name>"
```