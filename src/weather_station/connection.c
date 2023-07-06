/*This connection-progam tries to start a connection with the thies-clima-weather-station

importent: the SLAVE_ID for the inverter is 1; baudate: 9600; Bits 8N1; one register has 16bit or 4byte
-> only execution of 'Holding Register' with the implemented function 'modbus_read_registers(...)'
 
execution: ./connection <register> <size of reg>
[INFO]: only Holding Register possible */

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

void set_rts_custom(modbus_t*, int);
void config_gpio_pins();
void close_gpio_pins();

int main(int argc, char* argv[]){

    if (argc != 3){
        printf("usage: %s <register> <size of reg>\n[INFO]: only Holding Register possible\n", argv[0]);
        return -1;
    }

    const int REMOTE_ID = 1;
    int reg = atoi(argv[1]);
    int size_reg = atoi(argv[2]);

    modbus_t *ctx;
    uint16_t tab_reg[10]={0};

    config_gpio_pins();

    ctx = modbus_new_rtu("/dev/ttyS5", 9600, 'N', 8, 1);
    modbus_rtu_set_serial_mode(ctx, MODBUS_RTU_RS485);
    modbus_rtu_set_rts(ctx, MODBUS_RTU_RTS_UP);
    modbus_set_slave(ctx, REMOTE_ID);

    //Set custom function
    if (modbus_rtu_set_custom_rts(ctx, &set_rts_custom) == -1){
        fprintf(stderr, "Connection failed: %s\n", modbus_strerror(errno));
        modbus_free(ctx);
        return -1;
    }

    if (ctx == NULL) {
        fprintf(stderr, "Unable to create the libmodbus context\n");
        return -1;
    }

    if (modbus_connect(ctx) == -1) {
        fprintf(stderr, "Connection failed: %s\n", modbus_strerror(errno));
        modbus_free(ctx);
        return -1;
    }

    //Read register 
    if (modbus_read_registers(ctx, reg, size_reg, tab_reg) == -1){
        fprintf(stderr, "Connection failed: %s; reg: %d; reg-size: %d\n", modbus_strerror(errno),reg, size_reg);
        close_gpio_pins();
        modbus_free(ctx);
        return -1;
    }

     //Output read register
    for(int i=0;i<sizeof(tab_reg)/sizeof(tab_reg[0]);i++){
        printf("%d: %hu\n",i,tab_reg[i]);
    }

    modbus_free(ctx);
    close_gpio_pins();

    return 0;
}

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

    if (write(fd, "272", 3) != 3) {
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

    if (write(fd, "273", 3) != 3) {
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

}

void close_gpio_pins(){
    close(fd_272);
    close(fd_273);

    int fd = open("/sys/class/gpio/unexport", O_WRONLY);
    if (fd == -1) {
        perror("Unable to open /sys/class/gpio/unexport");
        exit(1);
    }
    if (write(fd, "273", 3) != 3) {
        perror("Error writing to /sys/class/unexport");
        exit(1);
    }
    if (write(fd, "272", 3) != 3) {
        perror("Error writing to /sys/class/unexport");
        exit(1);
    }

    close(fd);
}