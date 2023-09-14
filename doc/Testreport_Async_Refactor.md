# Testing Async
This is the testreport of the async refector of opensesame. (`-`) means that the test failed, (`~`) something went wrong but it works and (`+`) means that everything went as expected. Here is a list of the modules which need to be tested.

1. **Siganls** 
2. **Buttons (+Bell)**
3. **Buttons+Garage**
4. **Sensors**
5. **ModIR**
6. **Environment**
7. **Weatherstation**
8. **Battery**
9. **Watchdog**
10. **Ping**


## Signals
### Config
```toml
nextcloud.chat = "<chat-token>"
nextcloud.chat.licht = "<chat-token>"
nextcloud.chat.ping = "<chat-token>"
nextcloud.chat.commands = "<chat-token>"
nextcloud.user = "<user>"
nextcloud.pass = "<pass>"
nextcloud.url = "<nextcloud-url>"
nextcloud.format.datetime = "%d.%m.%Y %H:%M:%S"
nextcloud.format.time = "%H:%M:%S"
validator.test1 = "14, 15 ,13 ,15, 11, 15, 7, 15"
validator.test2 = "14, 12, 14, 15, 11, 15"
buttons.enable = 0
bell.enable = 0 
garage.enable = 0 
sensors.enable = 0
environment.enable = 0 
ir.enable = 0
weatherstation.enable = 0
bat.enable = 0
watchdog.enable = 0 
ping.enable = 0
```
### (`-`): This module doesn't enter the async-loop, because of problems with `spawn_local`

## Buttons
### Config 
```toml
...
buttons.enable = 1
...
```
### (`+`): light button; light switch; light permanent; light time extended; pin validation
### (`~`): First entering of pin caused time `sequence timeout`, the timeout was too fast. Besides this one failure it worked fine. Cloud be a startup issue.

## Buttons + Bell
### Config
```toml
...
buttons.enable = 1
bell.enable = 1
...
```
### (`+`): bell and buttons worked

## Buttons + Garage
### Config
```toml
...
buttons.enable = 1
garage.enable = 1
...
```
### (`+`): endposition garage door; TasterTorUnten (no NC message); TasterUnten (no NC message); TasterOben
### (`~`): TasterTorOben switched only the light indoor at the first time, after a retry it switch both

## Sensors
### Config
```toml
...
sensors.enable = 1
sensors.device = "/dev/ttyACM0"
sensors."#0".loc = "Wohnzimmer"
sensors."#0".alarm = 50
sensors."#0".chat = 20
sensors."#1".loc = "Badezimmer"
sensors."#1".alarm = 70
sensors."#1".chat = 35
sensors."#2".loc = "Küche"
sensors."#2".alarm = 5
sensors."#2".chat = 3
...
```
For simulating the sensors we used the methode with is described in [DevelopmentSetup](./DevelopmentSetup.md). With the following script:
```bash
COUNTER=0;
while true; do
    let COUNTER=COUNTER+1
    if [ $COUNTER -eq 20 ]; then
        data="23 35 4 0 0 0 0 0 0 0 0 0"
	elif [ $COUNTER -eq 100 ]; then
		data="55 80 10 0 0 0 0 0 0 0 0 0"
	elif [ $COUNTER -eq 130 ]; then 
		data="1005 2015 1102 0 0 0 0 0 0 0 0 0"
    else
        data="0 0 0 0 0 0 0 0 0 0 0 0"
    fi
    echo "$data"
    sleep 60
done
```
### (`+`): Chat and Alarm trigger worked as expected
### (`~`): **!!! implementation of state mutex is missing in async function of sensors !!!**