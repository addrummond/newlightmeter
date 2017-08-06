CC := gcc
override CFLAGS += -std=c99 -I./
override LDFLAGS += -lwiringPi -lrt

bluenrg.o: bluenrg.h bluenrg.c
	$(CC) $(CFLAGS) -c bluenrg.c -o bluenrg.o

test.o: test.c bluenrg.h
	$(CC) $(CFLAGS) -c test.c -o test.o

test: bluenrg.o test.o
	$(CC) $(LDFLAGS) bluenrg.o test.o -o test

.PHONY: clean
clean:
	rm *.o test
