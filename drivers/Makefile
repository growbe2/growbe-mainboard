TARGET = gro
#CC = arm-linux-gnueabihf-gcc
CC = gcc
CPPFLAGS = -I./include


mainboard_driver: main.o i2c_comboard.o i2c.o
	$(CC) $(CPPFLAGS) -o mainboard_driver main.o i2c_comboard.o i2c.o

main.o: main.c
	$(CC) $(CPPFLAGS) -c main.c

libmainboard_driver.a: i2c_comboard.o i2c.o
	ar rcs $@ $^

i2c_comboard.o: src/i2c_comboard.c include/i2c_comboard.h
	$(CC) $(CPPFLAGS) -c -o $@ $<

i2c.o: src/i2c.c include/i2c.h
	$(CC) $(CPPFLAGS) -c -o $@ $<

clean:
	rm -f *.o *.a $(TARGET)