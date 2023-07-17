# Weather Station
This documentation outlines the chosen steps for establishing communication with the CLIMA SENSOR US 4.920x.x0.xxx weather station. 
The hardware utilized for this communication includes an [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware) board with [LIME2-SHIELD](https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/open-source-hardware) and a [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module. 
Two command interpreters, namely THIES-ASCII and MODBUS RTU, can be chosen for communication, with THIES-ASCII being the default option. 

## important information 
According the [doc](https://www.vetterag.ch/images/pdf/thies/BA/4.920x.x0.xxx_ClimaSensor_US_d.pdf), the default `SLAVE_ID` is `0`, but in our specific case, it was configured as `1`.
Additionally, the documentation states that THIES-ASCII is the default interpreter, whereas our weather station was actually shipped with Modbus RTU.


## wiring of the RS-485 connection
According to the [doc](https://www.vetterag.ch/images/pdf/thies/BA/4.920x.x0.xxx_ClimaSensor_US_d.pdf), the wiring of the RS-485 connection should be done as follows:

1. Connect the GND (color: gray) of the weather station to the pull-down resistor of the [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module.
2. For the pins labeled `A` and `B`, it is important to connect all `A` pins together using a single wire, and similarly, connect all `B` pins together using a separate wire.
3. Referring to the documentation, there is a yellow wire labeled `+` and a green wire labeled `-`. Connect the A pin to the yellow `+` wire, and connect the green `-` wire to the B pin.

## supported Modbus functions
The libmodbus methode, which executes the different Modbus code function can be found in the [libmodbus-troubles.md](libmodbus-troubles.md) file.

### 0x03 - Read Holding Register
A listing of the registers of this function can be found in chapter 8.2.2 of the [doc]((https://www.vetterag.ch/images/pdf/thies/BA/4.920x.x0.xxx_ClimaSensor_US_d.pdf))
### 0x04 - Read Input Register 
A listing of the registers of this function can be found in chapter 8.2.1 of the [doc]((https://www.vetterag.ch/images/pdf/thies/BA/4.920x.x0.xxx_ClimaSensor_US_d.pdf))
### 0x06 - Write Single Register
### 0x10 - Write Mulriple Registers 
