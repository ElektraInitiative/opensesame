# [MOD-IR-TEMP](https://www.olimex.com/Products/Modules/Sensors/MOD-IR-TEMP/open-source-hardware) Module Description
## Setup and Wiring
The setup and wiring instructions are the same as those provided in the [Environment-documentation](./Environment.md)

## Configuration
### 1. Setting up Sensor Mode

### 2. Configuring [ir/enable], [ir/device], and [ir/data/interval]
You can configure the sensor by running the following commands: 
```bash
kdb set user:/sw/libelektra/opensesame/#0/current/ir/enable 1
kdb set user:/sw/libelektra/opensesame/#0/current/ir/device "/dev/ic2-2"
kdb set user:/sw/libelektra/opensesame/#0/current/ir/data/interval 60
```