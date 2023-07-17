# Goodwe inverter
This documentation outlines the chosen steps for establishing communication with a goodwe-inverter. 
The hardware utilized for this communication includes an [A20-OLinuXino-LIME2](https://www.olimex.com/Products/OLinuXino/A20/A20-OLinuXino-LIME2/open-source-hardware) board with [LIME2-SHIELD](https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/open-source-hardware) and a [MOD-RS485](https://www.olimex.com/Products/Modules/Interface/MOD-RS485/open-source-hardware) module. 

## important information
The `SLAVE_ID` by default `247`.
In the [doc](https://loxwiki.atlassian.net/wiki/spaces/LOX/pages/1605274474/Goodwe+GW10+ET+MODBUS+TCP+IP?preview=/1605274474/1605274552/Goodwe_Modbus_Protocol_Hybrid_ET_EH_BH_BT__ARM205%20HV__V1.7%20_%20Read%20Only_20200226%20(1).pdf) there is a list of all registers which can be read or writen.

## wiring of the RS-485 connection