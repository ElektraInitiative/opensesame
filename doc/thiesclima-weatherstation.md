# Weather Station
This documentation outlines the chosen steps for establishing communication with the CLIMA SENSOR US 4.920x.x0.xxx weather station. 
The hardware utilized for this communication includes an [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware) board with [LIME2-SHIELD](https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/open-source-hardware) and a [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module. 
Two command interpreters, namely THIES-ASCII and MODBUS RTU, can be chosen for communication, with THIES-ASCII being the default option. 
We will be utilizing the THIES-ASCII command interpreter.
Default ID on which every thiese weatherstation answers ist `99` so for detecting the device use `SLAVE_ID=99`. 

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

In our Rust code, we utilize the `serialport` and `gpio` modules. 
The `serialport` module allows us to send serial packages to the MOD-RS485 board, while the `gpio` module enables read and write operations by configuring the SCL/SCK (GPIO273) and #SS/SDA (GPIO272) pins. 
When both pins are set to `0`, the MOD-RS485 is ready to receive packages, and when both are set to `1`, the MOD-RS485 is ready to send packages.

Currently, we are able to observe output on the RS485 bus, but we have not received a response. This could be due to the weather station being set to full-duplex by default, whereas we can only use half-duplex.

To compile the Rust code, you need to install the `libudev` library. 
During cross-compilation, we encountered issues, so we opted to compile it directly on the [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware).

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

## Troubles
### MOD-RS485
When using the [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) in half-duplex mode, you have to switch SCK and #SS of the UEXT connection. 

### libmodbus-rs
## read_register timeout 
Because switching of SCK and #SS isn't enabled, so we need to use `rtu_set_custom_rts`, which is not implemented in [libmodbus-rs](https://github.com/zzeroo/libmodbus-rs) so wie need to programm this connection in `C`.
