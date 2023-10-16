# Environment sensors
## MOD-BME280

### Setup and Wiring 

#### 1. Enable i2c-2 by using `olinuxino-overlay` as root.

#### 2. Reboot to load new device tree.

#### 3. Used pins for i2c-2 on `GPIO-1 connector`.
These pins are the same as those used for UEXT1 on the [LIME2-SHIELD](https://www.olimex.com/Products/OLinuXino/A20/LIME2-SHIELD/open-source-hardware).
##### 3.1 For SCL, we used GPIO52 (PB20 on the A20-Chip) which is at pin `#32`.
##### 3.2 For SDA, we used GPIO53 (PB21 on the A20-Chip) which is at pin `#30`.
##### 3.3 For VDD, we used `3.3V`, which is at pin `#3`.
##### 3.4 For GND, we connected it to pin `#2`.

#### 5. The address of the BME-280 is at `0x77`, so we need to use the `BME280::new_secondary(i2c_bus, Delay);` function.
The following command reads the status register as a test:
```bash
/sbin/i2cget 2 0x77 0xF3
```