# Testing Async
This is the testreport of the async refector of opensesame. (`-`) means that the test failed, (`~`) something went wrong but it works and (`+`) means that everything went as expected. Here is a list of the modules which need to be tested.

1. **Signals** 
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
### Configuration
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
### (`+`): Is working and handles signals.

## Buttons
### Configuration 
```toml
...
buttons.enable = 1
...
```
### (`+`): light button; light switch; light permanent; light time extended; pin validation
### (`~`): First entering of pin caused time `sequence timeout`, the timeout was too fast. Besides this one failure it worked fine. Cloud be a startup issue.

## Buttons + Bell
### Configuration
```toml
...
buttons.enable = 1
bell.enable = 1
...
```
### (`+`): bell and buttons worked

## Buttons + Garage
### Configuration
```toml
...
buttons.enable = 1
garage.enable = 1
...
```
### (`+`): endposition garage door; TasterTorUnten (no NC message); TasterUnten (no NC message); TasterOben
### (`~`): TasterTorOben switched only the light indoor at the first time, after a retry it switch both

## Sensors
### Configuration
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

## Weatherstation
### Configuration
```toml
...
weatherstation.enable = 1
weatherstation.data.interval = 60
weatherstation.opensensemap.id = "64cb602193c69500072a580f"
weatherstation.opensensemap.token = "7bbb014ffbf974255caef2f88525b0512bd0817d9d222f70c7741a4a9cd56c6c"
...
```
### (`+`): Works as expected, sends warning to Nextcloud and updates opensensemap.org

## Buttons + Bell + PWR + Envirionment + Battery + Watchdog
### Configuration
```toml
nextcloud.chat = "<chat-default>"
nextcloud.chat.licht = "<chat-licht>"
nextcloud.chat.ping = "<chat-ping>"
nextcloud.chat.commands = "<chat-commands>"
nextcloud.user = "<user>"
nextcloud.pass = "<password>"
nextcloud.url = "https://nextcloud.markus-raab.org/nextcloud"
nextcloud.format.datetime = "%d.%m.%Y %H:%M:%S"
nextcloud.format.time = "%H:%M:%S"
validator.felix = "14, 15 ,13 ,15, 11, 15, 7, 15"
validator.sophie = "14, 12, 14, 15, 11, 15"
buttons.enable = 1 
bell.enable = 1
pwr.enable = 1 
environment.enable = 1 
bat.enable = 1 
watchdog.enable = 1
watchdog.path = "/dev/watchdog" 
watchdog.interval = 10
environment.device = "/dev/i2c-2"
environment.name = "Wohnzimmer"
environment.data.interval = 60
```
### Hardware Test and Documentation
#### Buttons

<center>

| **Pin-Buttons**      | **Result**   |
| -------------------- | ------------ |
| Validation            | `+` |
| Input Too Long        | `+` |
| Input Timeout         | `+` |

</center>
<center>

| **Light-Buttons**     | **Result**   |
| --------------------- | ------------ |
| Light extended        | `+` |
| Remove permanet Light | `+` |

</center>
<center>

| **Light-Taster** | **Result** |
| ---------------- | ---------- |
| Light extended   | `+` |
| Permanent Light  | `+` |
| Remove permanet Light | `+` |

</center>

#### Bell
<center>

| **Bell-Button** | **Result** |
| ---------------- | ---------- |
| Bell ringing   | `+` |

</center>
<center>

| **Bell-Taster** | **Result** |
| ---------------- | ---------- |
| Bell ringing   | `+` |

</center>

#### PWR
<center>

| **PWR reset** | **Result** |
| ---------------- | ---------- |
| do reset of MOD-IO2-Boards   | `+` |

</center>

#### Environment
<center>

| **Air Quality** | **Result** |
| ---------------- | ---------- |
| Information about environment data | `+` |

</center>

#### Battery
<center>

| **Check Capacity** | **Result** |
| ---------------- | ---------- |
| Waring if capacity is below 50% | `+` |

</center>

#### Watchdog
<center>

| **Stop triggering to watchdog** | **Result** |
| ---------------- | ---------- |
| Reboot | `+` |

</center>

## Buttons + Audio Bell + Garage + Battery + Watchdog
### Configuration
```toml
nextcloud.chat = "<chat-default>"
nextcloud.chat.licht = "<chat-licht>"
nextcloud.chat.ping = "<chat-ping>"
nextcloud.chat.commands = "<chat-commands>"
nextcloud.user = "<user>"
nextcloud.pass = "<password>"
nextcloud.url = "https://nextcloud.markus-raab.org/nextcloud"
nextcloud.format.datetime = "%d.%m.%Y %H:%M:%S"
nextcloud.format.time = "%H:%M:%S"
validator.felix = "14, 15 ,13 ,15, 11, 15, 7, 15"
validator.sophie = "14, 12, 14, 15, 11, 15"
buttons.enable = 1
audio.bell = "/home/olimex/bell_sound.ogg"
garage.enable = 1
bat.enable = 1 
watchdog.enable = 1
watchdog.path = "/dev/watchdog" 
watchdog.interval = 10
```
### Hardware Test and Documentation
#### Buttons
<center>

| **Pin-Buttons** | **Result**   |
| --------------- | ------------ |
| Validation 	  | `+` |
| Input Too Long  | `+` |
| Input Timeout   | `+` |

</center>
<center>

| **Light-Buttons**     | **Result**   |
| --------------------- | ------------ |
| Light extended        | `+` |
| Remove permanet Light | `+` |

</center>
<center>

| **Light-Taster** | **Result** |
| ---------------- | ---------- |
| Light extended   | `+` |
| Permanent Light  | `+` |
| Remove permanet Light | `+` |

</center>

#### Audio Bell
<center>

| **Audio Output** | **Result** |
| ---------------- | ---------- |
| Triggered by Nextcloud   | `+` |
| Triggered by Signals  | `+` |
| Triggered by Bell-Button | `+` |

</center>

#### Garage 

<center>

| **End position garage door** | **Result** |
| --------------------------- | ---------- |
| Information opened          | `+` |
| Information closed          | `+` |

</center>
<center>

| **TasterTorUnten and TasterUnten** | **Result** |
| ------------------ | ---------- |
| Open Door | `+` |

</center>
<center>

| **TasterTorOben**     | **Result** |
| --------------------- | ---------- |
| Switch both Lights on | `+` |

</center>
<center>

| **TasterOben**        | **Result** |
| --------------------- | ---------- |
| Switch inner Light on | `+` |

</center>

#### Battery
<center>

| **Check Capacity** | **Result** |
| ---------------- | ---------- |
| Waring if capacity is below 50% | `+` |

</center>

#### Watchdog
<center>

| **Stop triggering to watchdog** | **Result** |
| ---------------- | ---------- |
| Reboot | `+` |

</center>


## Weatherstation + Sensors + Audio Bell + Battery + Watchdog.
### Configuration
```toml
nextcloud.chat = "<chat-default>"
nextcloud.chat.licht = "<chat-licht>"
nextcloud.chat.ping = "<chat-ping>"
nextcloud.chat.commands = "<chat-commands>"
nextcloud.user = "<user>"
nextcloud.pass = "<password>"
nextcloud.url = "https://nextcloud.markus-raab.org/nextcloud"
nextcloud.format.datetime = "%d.%m.%Y %H:%M:%S"
nextcloud.format.time = "%H:%M:%S"
audio.bell = "/home/olimex/bell_sound.ogg"
bat.enable = 1
sensors.enable = 1
sensors.device = "/home/olimex/dev/ttyACM0"
sensors."#0".loc = "0"
sensors."#1".loc = "1"
sensors."#2".loc = "2"
sensors."#3".loc = "3"
sensors."#4".loc = "4"
sensors."#5".loc = "5"
sensors."#6".loc = "6"
sensors."#7".loc = "7"
sensors."#8".loc = "8"
sensors."#9".loc = "9"
sensors."#10".loc = "10"
sensors."#11".loc = "11"
weatherstation.enable = 1
weatherstation.opensensemap.id = "<opensensemap-box-id>" 
weatherstation.opensensemap.token = "<opensensemap-access-token>" 
weatherstation.data.interval = 60
watchdog.enable = 1
watchdog.path = "/dev/watchdog" 
watchdog.interval = 10

```
### Hardware Test and Documentation
#### Weatherstation
<center>

| **Warnings** | **Result** |
| ---------------- | ---------- |
| Warnings are send to Nextcloud | `+` |

</center>
<center>

| **Opensensemap** | **Result** |
| ---------------- | ---------- |
| Publish to Opensensemap | `+` |

</center>

#### Sensors
<center>

| **Warning and Alarms** | **Result** |
| ---------------- | ---------- |
| Information in Nextcloud Chat | `+` |

</center>

**Bash Script:**

```bash
#!/bin/bash

echo "152	237	279	275	177	166	90	440	59	370	423	9" 
sleep 60;
echo "153	237	279	258	177	166	106	441	81	370	429	22"
sleep 60;
echo "293	305	440	419	296	274	265	565	215	513	548	80"
sleep 60;
echo "340	349	505	426	356	369	364	628	344	576	594	145"
sleep 60;
echo "340	366	495	463	339	389	372	654	369	598	597	155"
sleep 60;
echo "371	388	514	465	348	395	392	676	410	625	618	180"
sleep 60;
echo "393	399	505	423	357	403	395	692	426	642	629	187"
sleep 60;
echo "359	395	491	414	352	391	357	693	370	639	595	144"
sleep 60;
echo "318	374	453	380	310	352	313	637	296	588	570	108"
sleep 60;
echo "304	365	439	368	298	328	283	618	244	563	562	82"
sleep 60;
echo "290	361	421	376	285	312	259	596	220	533	552	76"
sleep 60;
echo "270	353	393	347	264	278	218	571	169	487	536	53"
sleep 60;
echo "249	347	362	338	240	241	175	554	118	454	518	30"
sleep 60;
echo "236	342	343	341	228	227	160	545	103	442	507	25"
sleep 60;
echo "224	339	330	294	217	215	146	537	89	433	496	20"
sleep 60;
echo "216	335	318	304	209	207	138	533	83	429	489	18"
sleep 60;
echo "208	330	309	324	203	201	129	527	76	424	482	15"
sleep 60;
echo "201	326	301	275	197	195	122	524	73	423	477	14"

while true; do
	sleep 60;
	echo "196	330	303	298	192	190	115	517	69	416	469	12"
done
```
##### Audio Bell
<center>

| **Audio Output** | **Result** |
| ---------------- | ---------- |
| Triggered by Nextcloud   | `+` |
| Triggered by Signals  | `+` |

</center>

#### Battery
<center>

| **Check Capacity** | **Result** |
| ---------------- | ---------- |
| Waring if capacity is below 50% | `+` |

</center>

#### Watchdog
<center>

| **Stop triggering to watchdog** | **Result** |
| ---------------- | ---------- |
| Reboot | `+` |

</center>