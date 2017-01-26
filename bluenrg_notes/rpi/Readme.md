This dir contains some very rough-and-ready code for
the RPI 3B for testing the SPI interface with the 
BlueNRG. This doesn't make any attempt to implement
the protocol correctly, it just sends some initialization
commands and prints the responses.

Compile and run:

    gcc -O2 -std=c99 test.c -o test -lwiringPi -lrt && sudo ./test

