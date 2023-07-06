# libmodbus Trouble Protocol

## config modbus-rtu connection 
1. `ctx = modbus_new_rtu("/dev/ttyS5", 9600, 'N', 8, 1)` - set ttyS5 for serial communication; Baudrate to 9600; No paritybit; 8 databit; 1 stopbit
2. `modbus_rtu_set_serial_mode(ctx, MODBUS_RTU_RS485)`- set serial mode to RS485, other option would be RS323
3. `modbus_rtu_set_rts(ctx, MODBUS_RTU_RTS_UP)` - set the RTS-PINS (9&10) to high while sending, and while receiving low
4. `modbus_set_slave(ctx, REMOTE_ID)` - set the slave-id
5. `modbus_rtu_set_custom_rts(ctx, &set_rts_custom)` - set a custom function which is called before and after sending data, this is needed to set `DE` & `!RE` on the MOD-RS485 Module
6. `modbus_connect(ctx)` - starts the serial connection
7. `modbus_write_registers(ctx, reg, n, &write_data)` - write data to `reg` with the register size `n` and `write_data` will be written into the register

## methodes mapped to modbus-functions
### function 0x03 -> modbus_read_registers(...)

## weatherstation
### Date 05.07.2023 - execution of ./src/weather_station/connection
- `Connection timed out` with the above config and the `SLAVE_ID=0`
- `Illegal data value` with the above config and the `SLAVE_ID=1`
    - only using `modbus_write_registers(ctx, reg, n, &write_data)` with `SLAVE_ID=1` so wiring is ok
    
### Date 06.07.2023 - execution of ./src/goodwe-inverter/connection
The problem was that, we didn't use the right `SLAVE_ID`. The configured `SLAVE_ID` had been set to `1`.

## pv
### Date 05.07.2023
- `Connection timed out` with the above config and multiple `SLAVE_IS`s
### Date 06.07.2023
The problem was that, we didn't use the right `SLAVE_ID`. The configured `SLAVE_ID` had been set to `247`.

## `Connection timed out`reasons
- usage of wrong `SLAVE_ID`
- `DE`and `!RE`of MOD_RS485 is not set to `1` while sending or reset to `0` while receiving.  

## `Illegal data address`reasons
- usage of wrong `register address`