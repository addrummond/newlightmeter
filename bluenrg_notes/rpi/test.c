#include <wiringPi.h>
#include <wiringPiSPI.h>
#include <stdio.h>
#include <stdint.h>

static const uint8_t SERVER_BDADDR[] = {0x12, 0x34, 0x00, 0xE1, 0x80, 0x02};

static uint8_t buf[100];

static void zerobuf(unsigned l)
{
    for (unsigned i = 0; i < l && i < sizeof(buf)/sizeof(uint8_t); ++i)
        buf[i] = 0x00;
}

static void printbuf(unsigned l)
{
    printf("RESP: ");
    for (unsigned i = 0; i < l && i < sizeof(buf)/sizeof(uint8_t); ++i) {
        printf("%x ", buf[i]);
    }
    printf("\n");
}

int main()
{
    wiringPiSetup();
    pinMode(0, INPUT);
    pullUpDnControl(1, PUD_DOWN);
    pinMode(1, OUTPUT);

    digitalWrite(1, 0);
    delay(10);
    digitalWrite(1, 1);

    while (! digitalRead(0))
        ;

    printf("Got high IRQ following reset.\n");
    printf("Setting up SPI...\n");
    wiringPiSPISetup(0, 0);
    printf("SPI set up.\n");

    zerobuf(5);
    buf[0] = 0x0B;
    wiringPiSPIDataRW(0, buf, 5);
    printbuf(5);

    while (! digitalRead(0))
        ;

    zerobuf(5);
    buf[0] = 0x0B;
    wiringPiSPIDataRW(0, buf, 5);
    printbuf(5);

    while (! digitalRead(0))
        ;

    zerobuf(11);
    buf[0] = 0x0B;
    wiringPiSPIDataRW(0, buf, 11);
    printbuf(11);

    if (digitalRead(0))
        printf("STILL DATA TO READ??\n");

    zerobuf(17);
    buf[0] = 0x0A;
    buf[5] = 0x01;
    buf[6] = 0x0C;
    buf[7] = 0xFC;
    buf[8] = 8;
    buf[9] = 0;
    buf[10] = 6;
    buf[11] = SERVER_BDADDR[0];
    buf[12] = SERVER_BDADDR[1];
    buf[13] = SERVER_BDADDR[2];
    buf[14] = SERVER_BDADDR[3];
    buf[15] = SERVER_BDADDR[4];
    buf[16] = SERVER_BDADDR[5];
    wiringPiSPIDataRW(0, buf, 17);
    printbuf(17);

    while (! digitalRead(0));

    while (digitalRead(1)) {
        zerobuf(5);
        buf[0] = 0x0B;
        wiringPiSPIDataRW(0, buf, 5);
        printf("FIRST READ\n");
        printbuf(5);

        if (buf[3] == 7) {
            zerobuf(12);
            buf[0] = 0x0B;
            wiringPiSPIDataRW(0, buf, 12);
            printbuf(12);            
            break;
        }

        if (buf[3] != 0) {
            printf("UH OH!\n");
            break;
        }
    }

    return 0;
}
