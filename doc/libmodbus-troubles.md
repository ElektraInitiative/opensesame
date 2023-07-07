# libmodbus Trouble Protocol
The documentation of the libmodbus library can be found on [this website](https://libmodbus.org/reference/).
This brief troubleshooting protocol contains essential information required for communicating with a modbus device. It also includes details about the challenges we encountered during our programming journey.

## Methodes mapped to modbus-functions
The Modbus function codes provide a standardized set of commands for communication between Modbus devices.
### function 0x01 -> modbus_read_bits(modbus_t *ctx, int addr, int nb, uint8_t *dest)
### function 0x02 -> modbus_read_input_bits(modbus_t *ctx, int addr, int nb, uint8_t *dest) 
### function 0x03 -> modbus_read_registers(modbus_t *ctx, int addr, int nb, uint16_t *dest)
### function 0x04 -> modbus_read_input_registers(modbus_t *ctx, int addr, int nb, uint16_t *dest)
### function 0x05 -> modbus_write_bit(modbus_t *ctx, int addr, int status)
### function 0x06 -> modbus_write_register(modbus_t *ctx, int addr, const uint16_t value)
### function 0x0F -> modbus_write_bits(modbus_t *ctx, int addr, int nb, const uint8_t *src)
### function 0x10 -> modbus_write_registers(modbus_t *ctx, int addr, int nb, const uint16_t *src)

## Weatherstation
### Date 05.07.2023 - execution of ./src/weather_station/connection
- `Connection timed out` with the above config and the `SLAVE_ID=0`
- `Illegal data value` with the above config and the `SLAVE_ID=1`
    - only using `modbus_write_registers(ctx, reg, n, &write_data)` with `SLAVE_ID=1` so wiring is ok
    
### Date 06.07.2023 - execution of ./src/goodwe-inverter/connection
The problem was that, we didn't use the right `SLAVE_ID`. The configured `SLAVE_ID` had been set to `1`.

## Pv
### Date 05.07.2023
- `Connection timed out` with the above config and multiple `SLAVE_IS`s
### Date 06.07.2023
The problem was that, we didn't use the right `SLAVE_ID`. The configured `SLAVE_ID` had been set to `247`.

## Error: `Connection timed out`reasons
- usage of wrong `SLAVE_ID`
- `DE`and `!RE`of MOD_RS485 is not set to `1` while sending or reset to `0` while receiving.  

## Error: `Illegal data address`reasons
- usage of wrong `register address`