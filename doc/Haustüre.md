## GND

- blau mit rot
- transparent
- schwarz
- Abschirmung

## Taster 1

- LED gelb mit grün                -> Board20 GPIO4
- NC blau mit weiß/hellblau
- NO rosa                          -> Board20 GPIO0
- C gelb                           -> GND

## Taster 2

- LED rot mit schwarz              -> Board20 GPIO5
- NC orange
- NO braun                         -> Board20 GPIO1
- C blau                           -> GND

## Taster 3

- LED gelb mit schwarz             -> Board20 GPIO6
- NC violett
- NO braun mit rot                 -> Board20 GPIO2
- C rot mit weiß                   -> GND

## Taster 4

- LED hellgelb/hellbeige           -> Board21 GPIO4
- NC blau mit schwarz
- NO gelb mit rot                  -> Board20 GPIO3 (needs 47kΩ pull-up)
- C grün                           -> GND

## Taster Lampe

- LED weiß mit rot                 -> Board21 GPIO5
- NC orange mit rot
- NO grün mit rot                  -> Board21 GPIO0
- C orange mit weiß                -> GND

## Taster Bell/Glocke/Klingel

- LED braun mit weiß               -> Board21 GPIO6
- NC grün mit weiß
- NO violett mit weiß              -> Board21 GPIO1
- C gelb mit weiß                  -> GND


## LEDOutput

Insgesamt: 6

Mod_IO2: 6xOUT für LEDs
(Mod_IO: 4xOUT, davon 3x verwenden -> Mod_IO nicht gut geeignet!)


## sonstiger Input

- Taster bei Glocke außen (orange/weiß) -> Board21 GPIO0 (Taster Lampe)
- Taster Eingang Innen -> Board21 GPIO2
- Taster Glocke außen -> Board21 GPIO3 (needs 47kΩ pull-up)

Insgesamt: 9 (-1)

Mod_IO2: geht sich genau aus :-)
(Mod_IO: 4xIN mit opto-copling)



## Relay Output

- Türöffner 12V                   -> Board 20 Relay 1
- Aussen beim Eingang Licht       -> Board 20 Relay 2 (außen Snubber)
- Glocke Trafo 6V                 -> Board 21 Relay 1
- Innen Schuh LED Licht           -> Board 21 Relay 2 (kein Snubber!)


Info: Licht außen: Phase und Linie (Verbinden: Licht brennt)


## I2C

Sensor Medi

