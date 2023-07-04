#include <stdio.h>
#include <modbus.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

int fd_272;
int fd_273;

void set_rts_custom(modbus_t *ctx, int on){
    if (on){
        if (write(fd_272, "1", 1) != 1) {
            perror("Error writing to /sys/class/gpio/gpio272/value");
            exit(1);
        }
        if (write(fd_273, "1", 1) != 1) {
            perror("Error writing to /sys/class/gpio/gpio273/value");
            exit(1);
        }
    }else{
        if (write(fd_272, "0", 1) != 1) {
            perror("Error writing to /sys/class/gpio/gpio272/value");
            exit(1);
        }
        if (write(fd_273, "0", 1) != 1) {
            perror("Error writing to /sys/class/gpio/gpio273/value");
            exit(1);
        }
    }
}

void config_gpio_pins(){
     // Export the desired pin by writing to /sys/class/gpio/export
    int fd = open("/sys/class/gpio/export", O_WRONLY);
    if (fd == -1) {
        perror("Unable to open /sys/class/gpio/export");
        exit(1);
    }

    if (write(fd, "272", 2) != 2) {
        perror("Error writing to /sys/class/gpio/export");
        exit(1);
    }

    close(fd);

    // Set the pin to be an output by writing "out" to /sys/class/gpio/gpio24/direction

    fd = open("/sys/class/gpio/gpio272/direction", O_WRONLY);
    if (fd == -1) {
        perror("Unable to open /sys/class/gpio/gpio272/direction");
        exit(1);
    }

    if (write(fd, "out", 3) != 3) {
        perror("Error writing to /sys/class/gpio/gpio272/direction");
        exit(1);
    }

    close(fd);

    fd = open("/sys/class/gpio/export", O_WRONLY);
    if (fd == -1) {
        perror("Unable to open /sys/class/gpio/export");
        exit(1);
    }

    if (write(fd, "273", 2) != 2) {
        perror("Error writing to /sys/class/gpio/export");
        exit(1);
    }

    close(fd);

    // Set the pin to be an output by writing "out" to /sys/class/gpio/gpio24/direction

    fd = open("/sys/class/gpio/gpio273/direction", O_WRONLY);
    if (fd == -1) {
        perror("Unable to open /sys/class/gpio/gpio273/direction");
        exit(1);
    }

    if (write(fd, "out", 3) != 3) {
        perror("Error writing to /sys/class/gpio/gpio273/direction");
        exit(1);
    }

    close(fd);
}

int main(void){
    const int REMOTE_ID = 0;
    modbus_t *ctx;
    uint16_t tab_reg[32];

    config_gpio_pins();

    fd_273 = open("/sys/class/gpio/gpio273/value", O_WRONLY);
    if (fd_273 == -1) {
        perror("Unable to open /sys/class/gpio/gpio273/value");
        exit(1);
    }

    fd_272 = open("/sys/class/gpio/gpio272/value", O_WRONLY);
    if (fd_272 == -1) {
        perror("Unable to open /sys/class/gpio/gpio272/value");
        exit(1);
    }

    ctx = modbus_new_rtu("/dev/ttyS5", 9600, 'N', 8, 1);
    modbus_rtu_set_serial_mode(ctx, MODBUS_RTU_RS485);
    modbus_rtu_set_rts(ctx, MODBUS_RTU_RTS_UP);

    if (ctx == NULL) {
        fprintf(stderr, "Unable to create the libmodbus context\n");
        return -1;
    }

    if (modbus_connect(ctx) == -1) {
        fprintf(stderr, "Connection failed: %s\n", modbus_strerror(errno));
        modbus_free(ctx);
        return -1;
    }

    modbus_set_slave(ctx, REMOTE_ID);

    if (modbus_rtu_set_custom_rts(ctx, &set_rts_custom) == -1){
        fprintf(stderr, "Connection failed: %s\n", modbus_strerror(errno));
        modbus_free(ctx);
        return -1;
    }

    // Read 2 registers from address 0 of server ID 10.
    modbus_read_registers(ctx, 0, 2, tab_reg);

}