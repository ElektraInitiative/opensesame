# Weather Station
This documentation outlines the chosen steps for establishing communication with the CLIMA SENSOR US 4.920x.x0.xxx weather station. 
The hardware utilized for this communication includes an [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware) board with [LIME2-SHIELD](https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/open-source-hardware) and a [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module. 
Two command interpreters, namely THIES-ASCII and MODBUS RTU, can be chosen for communication, with THIES-ASCII being the default option. 
We will be utilizing the THIES-ASCII command interpreter.
Default ID on which every thiese weatherstation answers ist `99` so for detecting the device use `SLAVE_ID=99`.

## wiring of the RS-485 connection
According to the [documentation](https://www.vetterag.ch/images/pdf/thies/BA/4.920x.x0.xxx_ClimaSensor_US_d.pdf), the wiring of the RS-485 connection should be done as follows:

1. Connect the GND (color: gray) of the weather station to the pull-down resistor of the [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module.
2. For the pins labeled `A` and `B`, it is important to connect all `A` pins together using a single wire, and similarly, connect all `B` pins together using a separate wire.
3. Referring to the documentation, there is a yellow wire labeled `+` and a green wire labeled `-`. Connect the A pin to the yellow `+` wire, and connect the green `-` wire to the B pin.

## Initial Communication Steps

### Pin Identification
We connect MOD-RS485 via [UEXT](https://www.olimex.com/Products/Modules/) on UEXT2 of the LIME2- Shield (UEXT1 can be used for MOD-ENV).
The `TX` and `RX` pins of `UEXT2` are connected to the `uart7` of the A20-SoC.
In the next step, we searched the device tree for `uart7 (serial@1c29c00)` to determine its address.
By checking the system logs using `dmesg | grep '1c29c00'`, we found that `uart7` is mapped to `/dev/ttyS5`.

### Using mbpoll
Initially, we attempted to communicate with the [mbpoll](https://github.com/epsilonrt/mbpoll) tool. 
However, we encountered an issue where the weather station had the default address `0x00`, and mbpoll was unable to send Modbus RTU messages to this address. 
To resolve this, we found a solution on [GitHub](https://github.com/epsilonrt/mbpoll/issues/39), but we haven't tested it. Instead we wrote our own C-Code.


### Developing Custom Rust Code
#### ASCII
The initial attempt was to send ASCII messages over the serial connection, as implemented in the `src/weather_station/connection_ascii.rs` file. 
In the `connection_ascii.rs` file, we attempted to send data to configure the weather station for half-duplex usage. 
Although we were able to cache the output on the RS-485 bus, we did not receive a response from the weather station. 
This could be due to the weather station being configured with Modbus-RTU.

#### RTU-Modbus
The second attempt involved sending data using the libmodbus library, as implemented in the `src/weather_station/connection_modbus.rs` file. 
For this Modbus connection, we utilized [libmodbus-rs](https://github.com/zzeroo/libmodbus-rs), which provides an implementation of most of the methods from libmodbus. 
However, we encountered an issue while using [libmodbus-rs](https://github.com/zzeroo/libmodbus-rs), as the `rtu_set_custom_rts` function was not implemented. 
Therefore, we raised [issue #18](https://github.com/zzeroo/libmodbus-rs/issues/18). 
To overcome this problem, our plan is to temporarily program this communication in `C`.

### Using minicom
Another approach we considered was utilizing minicom for serial communication since it provides additional options for RS-485. 
However, we encountered the limitation of being unable to configure the SCL/SCK and #SS/SDA pins in minicom.

