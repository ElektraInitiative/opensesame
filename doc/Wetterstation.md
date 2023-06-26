# Weather Station
This documentation outlines the chosen steps for establishing communication with the CLIMA SENSOR US 4.920x.x0.xxx weather station. 
The hardware utilized for this communication includes an [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware) board with [LIME2-SHIELD](https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/open-source-hardware) and a [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module. 
Two command interpreters, namely THIES-ASCII and MODBUS RTU, can be chosen for communication, with THIES-ASCII being the default option. 
We will be utilizing the THIES-ASCII interpreter for this project.

## Initial Communication Steps

### Pin Identification
We used the `UEXT2` pins on the LIME2-Shield.
The `TX` and `RX` pins of `UEXT2` are connected to the `uart7` of the A20-SoC.
In the next step, we searched the device tree for `uart7 (serial@1c29c00)` to determine its address.
By checking the system logs using `dmesg | grep '1c29c00'`, we found that `uart7` is mapped to `/dev/ttyS5`.

### Using mbpoll
Initially, we attempted to communicate with the [mbpoll](https://github.com/epsilonrt/mbpoll) tool. 
However, we encountered an issue where the weather station had the default address `0x00`, and mbpoll was unable to send Modbus RTU messages to this address. 
To resolve this, we had to make some [minor modifications](https://github.com/epsilonrt/mbpoll/issues/39) to the source code of mbpoll and recompile it, enabling communication with address `0x00`.

### Developing Custom Rust Code

In our Rust code, we utilize the `serialport` and `gpio` modules. 
The `serialport` module allows us to send serial packages to the MOD-RS485 board, while the `gpio` module enables read and write operations by configuring the SCL/SCK (GPIO273) and #SS/SDA (GPIO272) pins. 
When both pins are set to `0`, the MOD-RS485 is ready to receive packages, and when both are set to `1`, the MOD-RS485 is ready to send packages.

Currently, we are able to observe output on the RS485 bus, but we have not received a response. This could be due to the weather station being set to full-duplex by default, whereas we can only use half-duplex.

To compile the Rust code, you need to install the `libudev` library. 
During cross-compilation, we encountered issues with the installation of libudev-armhf on hostsystem, so we opted to compile it directly on the [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware).

### Using minicom
Another approach we considered was utilizing minicom for serial communication since it provides additional options for RS-485. However, we encountered the limitation of being unable to configure the SCL/SCK and #SS/SDA pins in minicom.
