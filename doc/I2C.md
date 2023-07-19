# Important

- larger AC loads (>>10W) might need snubber circuits
- larger DC loads might need a diode
- avoid switching relays faster than every ~200ms
- avoid snubber parallel to switch on small loads
- avoid multiple appliances that might create interferences on the same board (i.e. connect larger loads to different mod-io2)
- snubber as near to load as possible

If all of this fails, reduce the I2C speed by activating the overlay using `sudo olinuxino-overlay` after installing Opensesame or manually with the file:
f3988a26e10c7b7f472284635a041c7e  i2c_2_clock_freq_6000_overlay.dtbo
(adding /usr/lib/olinuxino-overlays/sun7i-a20/i2c_2_clock_freq_6000_overlay.dtbo to `fdtoverlays` in `/boot/uEnv.txt` or use `sudo olinuxino-overlay`)

# Bus

with shield UEXT1 corresponds to /dev/i2c-2

# Devices

## Strip

LED DRIVER XY12J-1201000H-EW (12V output)
20mA Einschaltstrom
15mA Dauer (0,015A)
3,5W

## Bulb

Philipps LED E27 socket
13W (auch gemessen)
0.06A

## Panel

0.15A
37W

## All

Strip&Bulb&Panel Together

0.23A
50W


## 1

Board: Mod-IO2
Relay1 -> All

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.02s
CPU load during execution: 0.023320895% user, 0% nice, 8.762455% system, 0% intr, 91.214226% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0.13 0.41 0.21, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46.5

## 2

Board: Mod-IO2
Relay1 -> All, powered off
Relay2 -> Bulb, powered off

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.01s
CPU load during execution: 0.1398611% user, 0% nice, 9.6398735% system, 0% intr, 90.1502% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0.22 0.17 0.15, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46.8

## 4

Board: Mod-IO2
Relay1 -> All
Relay2 -> Bulb

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.04s
CPU load during execution: 0% user, 0% nice, 9.193275% system, 0% intr, 90.806725% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0.15 0.15 0.14, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46.4

## 5

Board: Mod-IO2
Relay1 -> Strip
Relay2 -> Bulb

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.01s
CPU load during execution: 0.023245003% user, 0% nice, 8.439959% system, 0% intr, 91.5368% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0 0.07 0.11, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47

## 6

Board: Mod-IO2
Relay1 -> Strip
Relay2 -> Bulb with 220nF Snubber (directly at relay)

From 50 iterations and 2100 tests I got 2 errors (get: 2 relay: 0)
Elapsed Time: 22.02s
CPU load during execution: 0% user, 0% nice, 9.189065% system, 0% intr, 90.81093% idle 
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0.12 0.03 0.06, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47
Following errors occurred:
("get", 16, 653, 15, 1)
("get", 37, 1535, 36, 1)

## 7

Board: Mod-IO2
Relay1 -> Strip with 220nF Snubber phase/null
Relay2 -> Bulb

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.04s
CPU load during execution: 0% user, 0% nice, 9.023613% system, 0% intr, 90.97639% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0 0.01 0.03, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47.3

## 8

Board: 2xMod-IO2 and Mod-IO connected on one bus

nothing connected to 0x21 board (which was running the tests)

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 21.96s
CPU load during execution: 0.023266636% user, 0% nice, 9.477938% system, 0% intr, 90.498795% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0.13 0.03 0.01, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 48.1

## 9

Board: 2xMod-IO2 and Mod-IO connected on one bus
Relay1 -> Strip with 220nF Snubber phase/null
Relay2 -> Bulb

From 50 iterations and 2100 tests I got 1 errors (get: 1 relay: 0)
Elapsed Time: 22.00s
CPU load during execution: 0.023277467% user, 0% nice, 9.132166% system, 0% intr, 90.84456% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0.1 0.09 0.03, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 48.4
Following errors occurred:
("get", 12, 485, 11, 1)

## 10

Board: 2xMod-IO2 and Mod-IO connected on one bus
Relay1 -> Strip (now without Snubber)
Relay2 -> Bulb

From 50 iterations and 2100 tests I got 2 errors (get: 2 relay: 0)
Elapsed Time: 22.06s
CPU load during execution: 0.04672947% user, 0% nice, 9.042841% system, 0% intr, 90.91043% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0 0.04 0.01, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 48.1
Following errors occurred:
("get", 19, 779, 18, 1)
("get", 33, 1367, 32, 1)

## 11

Board: 2xMod-IO2 and Mod-IO connected on one bus
Relay1 -> All
Relay2 -> Bulb

From 50 iterations and 2100 tests I got 4 errors (get: 4 relay: 0)
Elapsed Time: 22.10s
CPU load during execution: 0.04653343% user, 0% nice, 9.328784% system, 0% intr, 90.62468% idle
Executed version 0.4.43 with boot time 2022-07-25 04:27:52 UTC
Load average: 0 0.03 0, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47.8
Following errors occurred:
("get", 13, 527, 12, 1)
("get", 34, 1409, 33, 1)
("get", 40, 1661, 39, 1)
("get", 43, 1787, 42, 1)

## 12

Board: Mod-IO2 20; Mod-IO 58; Mod-IO2 21 connected on one bus
Board Mod-IO2 0x20 used
Relay1 -> All with Snubber (between phase+null)
Relay2 -> Bulb with Snubber (between phase+null)

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.04s
CPU load during execution: 0% user, 0% nice, 9.489354% system, 0% intr, 90.51064% idle 
Executed version 0.4.43 with boot time 2022-07-25 09:45:58 UTC
Load average: 0 0.01 0.06, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47.5

## 13

Board: Mod-IO2 20; Mod-IO 58; Mod-IO2 21 connected on one bus
Board Mod-IO2 0x20 used
Relay1 -> All (no snubber)
Relay2 -> Bulb (no snubber)

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.04s
CPU load during execution: 0.023245003% user, 0% nice, 9.362755% system, 0% intr, 90.614% idle 
Executed version 0.4.43 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.19 0.06 0.06, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47.3

## 14

Board: Mod-IO 58; Mod-IO2 21; Mod-IO2 20 connected on one bus (i.e. order changed 0x20 is now last)
Board Mod-IO2 0x20 used
Relay1 -> All (no snubber)
Relay2 -> Bulb (no snubber)

From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 22.03s
CPU load during execution: 0.023201857% user, 0% nice, 9.828774% system, 0% intr, 90.14802% idle
Executed version 0.4.43 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.01 0.03 0.05, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 47.4


## 15

Mod-IO
nothing powered on

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 21.98s
CPU load during execution: 0.046448514% user, 0% nice, 10.05428% system, 0% intr, 89.89927% idle
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.31 0.31 0.19, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46
Debugging enabled


## 16

Mod-IO
Relay1-> All

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 6 errors (get: 6 relay: 0)
Elapsed Time: 21.99s
CPU load during execution: 0.092927374% user, 0% nice, 9.965683% system, 0% intr, 89.94139% idle
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.05 0.21 0.17, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.9
Debugging enabled
Following errors occurred:
("get", 2, 65, 1, 1)
("get", 5, 191, 4, 1)
("get", 9, 359, 8, 1)
("get", 29, 1199, 28, 1)
("get", 44, 1829, 43, 1)
("get", 46, 1913, 45, 1)

## 17

only doing get operations (without relay)

Using address 0x58, modio: true
From 50 iterations and 2000 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 21.92s
CPU load during execution: 0.02340824% user, 0% nice, 9.261242% system, 0% intr, 90.69174% idle 
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.5 0.31 0.2, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46.2
Debugging enable

## 18

Relay1 -> All using snubber

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 3 errors (get: 3 relay: 0)
Elapsed Time: 21.89s
CPU load during execution: 0.06945537% user, 0% nice, 10.620163% system, 0% intr, 89.31039% idle
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.65 0.36 0.23, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46.6
Following errors occurred:
("get", 32, 1325, 31, 1)
("get", 37, 1535, 36, 1)
("get", 50, 2081, 49, 1)

## 19

Relay1 -> All using snubber and Tyristor (lying on contact)

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 4 errors (get: 4 relay: 0)
Elapsed Time: 23.94s
CPU load during execution: 0.10607628% user, 0% nice, 7.84043% system, 0% intr, 92.0535% idle
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.55 0.37 0.24, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46
Following errors occurred:
("get", 13, 527, 12, 1)
("get", 15, 611, 14, 1)
("get", 42, 1745, 41, 1)
("get", 43, 1787, 42, 1)

## 20

Relay1 -> All using snubber (at relay contacts) and Tyristor (blue KC472M connected with phase/null)

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 1 errors (get: 1 relay: 0)
Elapsed Time: 21.88s
CPU load during execution: 0.023191094% user, 0% nice, 10.715207% system, 0% intr, 89.261604% idle 
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.14 0.18 0.18, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.8
Following errors occurred:
("get", 1, 1, 0, 0)

## 21

Relay1 -> All using snubber (at relay contacts) and ceramik Kondensator (22nF 250V HSK RX)

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 21.81s
CPU load during execution: 0.069984615% user, 0% nice, 10.375597% system, 0% intr, 89.55442% idle
Executed version 0.4.44 with boot time 2022-07-25 09:45:58 UTC
Load average: 0.12 0.03 0.07, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.9


## 22

Relay1 -> Strip+Bulb using snubber (at relay contacts), Power was off!

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 1 errors (get: 1 relay: 0)
Elapsed Time: 21.88s
CPU load during execution: 0.023320895% user, 0% nice, 10.336704% system, 0% intr, 89.63998% idle
Executed version 0.4.44 with boot time 2022-07-25 16:57:52 UTC
Load average: 0 0 0, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.2
Following errors occurred:
("get", 1, 1, 0, 0)

## 22

Relay1 -> Strip+Bulb using 2x snubber (at relay contacts+at lamps), Power was off!

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 1 errors (get: 1 relay: 0)
Elapsed Time: 21.87s
CPU load during execution: 0.046598323% user, 0% nice, 9.344325% system, 0% intr, 90.60907% idle
Executed version 0.4.44 with boot time 2022-07-25 16:57:52 UTC
Load average: 0.06 0.02 0, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.4
Following errors occurred:
("get", 1, 1, 0, 0)

## 23

Relay1 -> Strip+Bulb using 2x snubber (at relay contacts+at lamps), Power was ON!

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 1 errors (get: 1 relay: 0)
Elapsed Time: 21.96s
CPU load during execution: 0.02306273% user, 0% nice, 10.0900135% system, 0% intr, 89.886925% idle
Executed version 0.4.44 with boot time 2022-07-25 16:57:52 UTC
Load average: 0.05 0.02 0, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.5
Following errors occurred:
("get", 44, 1829, 43, 1)

## 24

Relay1 -> Strip+Bulb using snubber (at relay contacts) and ceramik Kondensator (22nF 250V HSK RX) at lamp, Power was ON!

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 2 errors (get: 2 relay: 0)
Elapsed Time: 21.89s
CPU load during execution: 0.04668534% user, 0% nice, 10.237276% system, 0% intr, 89.71604% idle
Executed version 0.4.44 with boot time 2022-07-25 16:57:52 UTC
Load average: 0 0 0, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.3
Following errors occurred:
("get", 1, 1, 0, 0)
("get", 30, 1241, 29, 1)

## 25

Relay1 -> All using snubber (at relay contacts) and ceramik Kondensator (22nF 250V HSK RX) at lamp, Power was ON!

Using address 0x58, modio: true
From 50 iterations and 2100 tests I got 2 errors (get: 2 relay: 0)
Elapsed Time: 21.91s
CPU load during execution: 0.023245003% user, 0% nice, 9.935416% system, 0% intr, 90.041336% idle
Executed version 0.4.44 with boot time 2022-07-25 16:57:52 UTC
Load average: 0 0 0, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 44.7
Following errors occurred:
("get", 41, 1703, 40, 1)
("get", 44, 1829, 43, 1)

Strip was not oscillating, so ceramik Kondensator makes a difference!










## Old stuff

### 1

Board: MOD-IO2
COM  -> 230V Phase
NO   -> LED DRIVER XY12J-1201000H-EW (12V output)

for i in `seq 1 5`; do;
/sbin/i2cset -y 2 0x21 0x40 0x03; /sbin/i2cset -y 2 0x21 0x10; /sbin/i2cget -y 2 0x21; sleep 0.1
/sbin/i2cset -y 2 0x21 0x40 0x00; /sbin/i2cset -y 2 0x21 0x10; /sbin/i2cget -y 2 0x21; sleep 0.2
done


-> works without any problems

### 2

COM und NO vertauscht

Board: MOD-IO2
NO   -> 230V Phase
COM  -> LED DRIVER XY12J-1201000H-EW (12V output)

-> works without any problems

### 3

with 0.047 µF 100Ω Snubber between NO und COM
-> immediately blinking


### 4

with 220nF 100Ω betwen NO und COM
-> immediately blinking (lower freq)

### 5

Board: MOD-IO2
Rel1:
NO   -> 230V Phase
COM  -> LED DRIVER XY12J-1201000H-EW (12V output)
Rel2:
NO   -> 230V Phase
COM  -> Light bulb

errors occur! "Error: Write failed"


### 6

Beide mit 22nF&100 ohm parallel zur Last (direkt an PINs der Lampen)

--> alles funktioniert!

nochmals mit 0.1 und 0.2 sleep:

errors occur! "Error: Write failed"


### 7

Beide mit 22nF&100 ohm direkt bei Relay und Nulleiter

--> Manchmal kaputt

### 8

Nur Glühlampe mit 220nF 100Ω geschützt (direkt bei Kontakt)


### 9

Relays einfach so (keine Last: strom war aus)

2 errors bei 10ms
0 errors bei 50ms

From 50 iterations and 1100 tests I got 23 errors (get: 22 relay: 1)
Elapsed Time: 12.89s
CPU load during execution: 0.5913285% user, 0% nice, 10.086831% system, 0% intr, 89.32184% idle 
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 1.25 0.59 0.23, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 49.5
Following errors occurred:
("get", 3, 62, 2, 6)
("get", 5, 106, 4, 6)
("get", 6, 123, 5, 1)
("get", 7, 139, 6, 6)
("get", 7, 153, 6, 9)
("get", 8, 171, 7, 5)
("get", 9, 189, 8, 1)
("get", 10, 207, 9, 8)
("relay", 10, 209, 9, 1)
("get", 11, 234, 10, 2)
("get", 15, 327, 14, 7)
("get", 16, 350, 15, 8)
("get", 18, 376, 17, 1)
("get", 18, 395, 17, 9)
("get", 21, 457, 20, 5)
("get", 28, 613, 27, 7)
("get", 38, 816, 37, 1)
("get", 40, 876, 39, 6)
("get", 41, 885, 40, 4)
("get", 45, 976, 44, 7)
("get", 45, 983, 44, 3)
("get", 47, 1015, 46, 2)
("get", 48, 1038, 47, 3)

Wenn allerdings gar nichts angesteckt:

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.86s
CPU load during execution: 0.51153415% user, 0% nice, 10.19187% system, 0% intr, 89.2966% idle 
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.12 0.37 0.19, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48



### 10

Mit Lampen bei 50ms

6 Fehler

From 100 iterations I got 29 errors (get: 28 relay: 1)
Elapsed Time: 15.00s
CPU load during execution: 0.7445322% user, 0% nice, 10.935025% system, 0% intr, 87.60828% idle
Executed version 0.4.40 with boot time 2022-07-21 10:15:03 UTC
Load average: 1.26 1.6 1.25, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 49.


### 11

Nur LED connected, ohne Strom

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.86s
CPU load during execution: 0.51077384% user, 0% nice, 11.015283% system, 0% intr, 88.47394% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.09 0.24 0.16, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48.1

### 12

Mit LED, mit Strom (with snubber)

From 50 iterations and 1100 tests I got 3 errors (get: 3 relay: 0)
Elapsed Time: 12.92s
CPU load during execution: 0.6661278% user, 0% nice, 10.001883% system, 0% intr, 89.33199% idle 
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.4 0.31 0.19, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 47.9
Following errors occurred:
("get", 3, 56, 2, 0)
("get", 29, 628, 28, 0)
("get", 39, 848, 38, 0)

again without snubber

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.87s
CPU load during execution: 0.669703% user, 0% nice, 10.792065% system, 0% intr, 88.53823% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.16 0.05 0.08, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48

with tyristor:

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.88s
CPU load during execution: 0.6279134% user, 0% nice, 10.326118% system, 0% intr, 89.04597% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.11 0.05 0.08, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48


### 13

Nochmals ohne Strom:

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.86s
CPU load during execution: 0.51169074% user, 0% nice, 10.87159% system, 0% intr, 88.616714% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.22 0.28 0.18, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48

### 14

Nur Lampe connected, ohne Strom

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.85s
CPU load during execution: 0.62920004% user, 0% nice, 11.008681% system, 0% intr, 88.36212% idle 
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.19 0.16 0.15, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48.1

### 15

Mit Lampe und Strom:

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.86s
CPU load during execution: 0.62924343% user, 0% nice, 11.014824% system, 0% intr, 88.355934% idle 
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.16 0.15 0.15, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48.1

### 16

Lampe mit Snubber, LED mit Tyristor

From 50 iterations and 1100 tests I got 1 errors (get: 1 relay: 0)
Elapsed Time: 12.87s
CPU load during execution: 0.6304317% user, 0% nice, 10.086907% system, 0% intr, 89.28265% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.02 0.04 0.07, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48
Following errors occurred:
("get", 40, 870, 39, 0)


### 17

LED-Lampe (Philipps) mit Snubber, LED Driver mit Tyristor
OHNE Strom

From 50 iterations and 1100 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 12.89s
CPU load during execution: 0.7064403% user, 0% nice, 10.126014% system, 0% intr, 89.16755% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0 0.02 0.06, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 47.9

### 18

wie 17, mit Strom

From 50 iterations and 1100 tests I got 2 errors (get: 2 relay: 0)
Elapsed Time: 12.88s
CPU load during execution: 0.66815% user, 0% nice, 10.9675045% system, 0% intr, 88.36434% idle
Executed version 0.4.41 with boot time 2022-07-21 10:15:03 UTC
Load average: 0.16 0.05 0.06, Memory usage: 1037.6 MB, Swap: 0 B, CPU temp: 48
Following errors occurred:
("get", 2, 34, 1, 0)
("get", 28, 606, 27, 0)


### 19

Completely without any relays

From 14400 iterations and 14400 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 7.72s
CPU load during execution: 0.07331378% user, 0% nice, 2.0049725% system, 0% intr, 97.921715% idle
Executed version 0.4.42 with boot time 2022-07-23 04:31:09 UTC
Load average: 1.35 0.83 0.35, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 46.8

From 14400000 iterations and 14400000 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 7703.56s
CPU load during execution: 0.13824892% user, 0% nice, 1.1674037% system, 0% intr, 98.66128% idle
Executed version 0.4.42 with boot time 2022-07-23 04:31:09 UTC
Load average: 1 1 1, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.9

From 43200000 iterations and 43200000 tests I got 0 errors (get: 0 relay: 0)
Elapsed Time: 23104.37s
CPU load during execution: 0.112530194% user, 0% nice, 0.97317564% system, 0% intr, 98.896286% idle
Executed version 0.4.42 with boot time 2022-07-23 04:31:09 UTC
Load average: 1 1 1, Memory usage: 1046.7 MB, Swap: 0 B, CPU temp: 45.7


# Allgemeine Infos

---------

mod-IO2:

## close jumper and execute
./modio2tool --setaddress 0x20 # works!
/sbin/i2cset 2 0x21 0x22 #doesn't work
## to rename board to 0x22


relay ein:

--------

mod-IO:

i2cset -y 2 0x58 0x10 0x01
i2cset -y 2 0x58 0x10 0x00

## read IN1 AIN-2:
i2ctransfer -y 2 w1@0x58 0x20 r1



---------

sudo adduser olimex i2c
sudo chown root.i2c /dev/gpiochip*

## strace /sbin/i2cset -y -f 2 0x58 0x10 0x00

openat(AT_FDCWD, "/dev/i2c-2", O_RDWR)  = 3
ioctl(3, _IOC(_IOC_NONE, 0x7, 0x5, 0), 0xbece468c) = 0
ioctl(3, _IOC(_IOC_NONE, 0x7, 0x6, 0), 0x58) = 0
ioctl(3, _IOC(_IOC_NONE, 0x7, 0x20, 0), 0xbece4600) = 0
close(3)                                = 0
exit_group(0)                           = ?

## strace /sbin/i2cset -y -f 2 0x58 0x10 0x0F

openat(AT_FDCWD, "/dev/i2c-2", O_RDWR)  = 3
ioctl(3, _IOC(_IOC_NONE, 0x7, 0x5, 0), 0xbea2068c) = 0
ioctl(3, _IOC(_IOC_NONE, 0x7, 0x6, 0), 0x58) = 0
ioctl(3, _IOC(_IOC_NONE, 0x7, 0x20, 0), 0xbea20600) = 0
close(3)                                = 0
exit_group(0)                           = ?


https://docs.rs/i2cdev/0.5.1/i2cdev/
