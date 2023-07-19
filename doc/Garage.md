# Allgemeine Infos

- Lampe übern Garagentor: braun, blau, gelb/grün
- Schalter Endstellung Garagentor: braun COM; schw NO; grau NC; gelb/grün am tor
- Taster bei Eingang: oben: orange und weiß; unten: blau und weiß
- Board21 GPIO3 is unused and needs to be connected to +5V (due to non-working internal pull-up)

# GPIO Input

Layout Platine

- Taster Eingang Oben             -> Pin40 GPIO234 EINT10
- Taster Eingang Unten            -> Pin38 GPIO235 EINT11
- Taster Tor Oben                 -> Pin36 GPIO236 EINT12
- Taster Tor Unten                -> Pin32 GPIO237 EINT13
- Schalter Garagentor Endposition -> Pin26 GPIO238 EINT14


# LEDS

- LED_1 "1"     black/violet       -> Board20 GPIO4
- LED_2 "2"     salmonorange       -> Board20 GPIO5
- LED_3 "3"     red/black          -> Board20 GPIO6
- LED_4 "4"     dark turquise      -> Board21 GPIO4
- LED_L "Light" cow pie brown      -> Board21 GPIO5
- LED_B "Bell"  baby-blue          -> Board21 GPIO6

# BUTTONS

- BUTTON_1 "1"      zitronengelb                    -> Board20 GPIO0
- BUTTON_2 "2"      blassrosa                       -> Board20 GPIO1
- BUTTON_3 "3"      dunkeltürkis/schwarz            -> Board20 GPIO2
- BUTTON_4 "4"      helltürkis                      -> Board20 GPIO3 (needs 47k pull-up)
- BUTTON_L "Light"  weiß                            -> Board21 GPIO0
- BUTTON_B "Bell"   dunkelblau                      -> Board21 GPIO1

# GND

"C" (ground from buttons)

- grau/schwarz
- gelb/schwarz
- weiß/schwarz
- hellblau/schwarz
- blassrosa/schwarz
- dunkelblau/schwarz

# not connected: (NC)

- rosa/schwarz
- rosaorange/schwarz
- schwarz
- helltürkis/schwarz
- rot
- violett


# Relay Output

- Garagenöffner                   -> Board 20 Relay 1
- Aussen Licht                    -> Board 20 Relay 2
- (keine Glocke)                  -> Board 21 Relay 1
- Innen Panel Licht               -> Board 21 Relay 2



