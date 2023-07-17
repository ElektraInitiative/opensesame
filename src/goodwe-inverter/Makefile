all: connection.c
	gcc connection.c `pkg-config --cflags --libs libmodbus` -o connection

clean:
	$(RM) connection
	