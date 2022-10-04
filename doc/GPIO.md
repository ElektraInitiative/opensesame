## used GPIOs for PINs

"1"     GPIO53  P3  black/violet
"2"     GPIO52  P5  salmonorange
"3"     GPIO271 P7  red/black
"4"     GPIO267 P11 dark turquise
"Light" GPIO266 P13 cow pie brown
"Bell"  GPIO263 P15 baby-blue


echo "52"  > /sys/class/gpio/export
echo "53"  > /sys/class/gpio/export
echo "271" > /sys/class/gpio/export
echo "267" > /sys/class/gpio/export
echo "266" > /sys/class/gpio/export
echo "263" > /sys/class/gpio/export
echo "85"  > /sys/class/gpio/export
echo "86"  > /sys/class/gpio/export

echo out > /sys/class/gpio/gpio53/direction
echo out > /sys/class/gpio/gpio52/direction
echo out > /sys/class/gpio/gpio271/direction
echo out > /sys/class/gpio/gpio267/direction
echo out > /sys/class/gpio/gpio266/direction
echo out > /sys/class/gpio/gpio263/direction
echo in > /sys/class/gpio/gpio85/direction
echo in > /sys/class/gpio/gpio86/direction

echo 1 > /sys/class/gpio/gpio53/value
echo 1 > /sys/class/gpio/gpio52/value
echo 1 > /sys/class/gpio/gpio271/value
echo 1 > /sys/class/gpio/gpio267/value
echo 1 > /sys/class/gpio/gpio266/value
echo 1 > /sys/class/gpio/gpio263/value
cat /sys/class/gpio/gpio85/value
cat /sys/class/gpio/gpio86/value

chown -R olimex.olimex /sys/class/gpio
chown -R olimex.olimex /sys/devices/platform/soc/1c20800.pinctrl/gpiochip0



echo "52"  > /sys/class/gpio/unexport
echo "53"  > /sys/class/gpio/unexport
echo "271" > /sys/class/gpio/unexport
echo "267" > /sys/class/gpio/unexport
echo "266" > /sys/class/gpio/unexport
echo "263" > /sys/class/gpio/unexport
echo "85"  > /sys/class/gpio/unexport
echo "86"  > /sys/class/gpio/unexport


## calculate GPIO numbers on A20 0.5''

PA, PB, PC, PD, PE, PF, PG, PH, PI

GPIO ports are numbered from 0 to 287

gpioNumber = (Port Letter - 'A') * 32 + pinNumber

PH2:
H-A = 7*32=224
224+2 = 226


https://olimex.wordpress.com/2019/01/25/working-with-a20-olinuxino-or-som-gpios-when-using-new-armbian-based-a20-universal-linux-image/


## debug GPIO

cat /sys/kernel/debug/gpio

