#include <wiringPi.h>
#include <wiringPiSPI.h>
#include <stdio.h>
#include <stdint.h>

#define RESET_PIN 1
#define IRQ_PIN   0

static const uint8_t SERVER_BDADDR[] = {0x12, 0x34, 0x00, 0xE1, 0x80, 0x02};

static uint8_t buf[100];

static void zerobuf(unsigned l)
{
    while (! digitalRead(IRQ_PIN))
        ;
}

static void printbuf(unsigned l)
{
    // Seems to be occasional bug with timeout based on CLOCK_REALTIME,
    // so we also use a loop count timeout (which should last longer).

    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) < 0)
        return -1;
    long t1n = ts.tv_nsec;

    for (unsigned i = 0; i < NORMAL_LOOP_COUNT_TIMEOUT; ++i) {
        if (digitalRead(IRQ_PIN))
            return 1;
        
        if (clock_gettime(CLOCK_REALTIME, &ts) < 0)
            return -1;
        
        long diff = ts.tv_nsec - t1n;
        if (diff > NORMAL_TIMEOUT_NS)
            return 0;
    }

    return 0;
}

int irq_is_high()
{
    return digitalRead(IRQ_PIN);
}

int wait_to_write_n(unsigned n)
{
    unsigned write = 0;
    while (write < n) {
        int r = poll_read_write(NULL, &write);
        if (r < 0)
            return r;
    }
    return 0;
}

int read_n_bytes(unsigned offset, uint8_t rbuf[], unsigned n)
{
    uint8_t buf[n + 5];
    unsigned start = 0;
    for (unsigned i = 0; start < n; ++i) {
        // Timeout logic.
        if (i > n * READ_LOOP_MAX_COUNTS_PER_BYTE)
            return -3;

        buf[start] = 0x0B;
        for (unsigned i = start+1; i < n + 5 - start; ++i)
            buf[i] = 0;

        int r = wiringPiSPIDataRW(start, buf, n + 5 - start);
        if (r < 0)
            return r;
        
        if (buf[0] != 2) {
            fprintf(stderr, "Unexpected packet code in read_n_bytes.\n");
            return -1;
        }

        start += buf[3] + (((unsigned)buf[4]) << 8);
    }

    for (unsigned i = 0; i < n; ++i) {
        rbuf[i] = buf[i + 5];
    }

    return 0;
}

int send_command(unsigned command, const Param params[], unsigned n_params)
{
    assert(params != NULL || n_params == 0);

    unsigned parm_total_length = 0;
    for (unsigned i = 0; i < n_params; ++i) {
        parm_total_length += params[i].length;
    }
    unsigned buflen = parm_total_length + 9;

    int r = wait_to_write_n(parm_total_length + 1);
    if (r < 0)
        return r;

    uint8_t buf[buflen];

    unsigned bufi = 9;
    for (unsigned i = 0; i < n_params; ++i) {
        memcpy(buf + bufi, params[i].data, params[i].length);
        bufi += params[i].length;
    }

    buf[0] = 0x0A;
    buf[1] = 0x00;
    buf[2] = 0x00;
    buf[3] = 0x00;
    buf[4] = 0x00;
    buf[5] = 0x01; // Mark following as command packet
    buf[6] = command & 0xFF;
    buf[7] = command >> 8;
    buf[8] = parm_total_length;

    printf("WRITING: ");
    for (unsigned i = 0; i < buflen; ++i)
        printf("%02x ", buf[i]);
    printf("\n");

    r = wiringPiSPIDataRW(0, buf, buflen);
    if (r < 0)
        return r;
    
    return 0;
}

int send_command_and_get_response(unsigned command, const Param params[], unsigned n_params, uint8_t response[], unsigned max_response_len)
{
    assert(response != NULL || max_response_len == 0);

    for (;;) {
        int r = send_command(command, params, n_params);
        if (r < 0)
            return r;

        r = wait_for_irq_with_timeout();
        if (r < 0)
            return r;
        if (r == 1)
            break;
    }

    unsigned max_status_len = max_response_len + 6;
    
    unsigned total_read = 0;
    uint8_t status_buf[max_status_len];

    for (unsigned i = 0; irq_is_high(); ++i) {
        // Timeout logic.
        if (i > READ_LOOP_MAX_COUNTS_PER_BYTE)
            return -3;

        unsigned read;
        int r = poll_read_write(&read, NULL);
        if (r < 0)
            return r;
        
        if (total_read + read > max_status_len)
            return -2;
                
        if (read > 0) {
            r = read_n_bytes(total_read, status_buf, read);
            if (r < 0)
                return r;
            
            total_read += read;
        }
    }

    printf("STATBUF: ");
    for (unsigned i = 0; i < total_read; ++i) {
        printf("%02x ", status_buf[i]);
    }
    printf("\n");
}

int main()
{
    wiringPiSetup();
    pinMode(IRQ_PIN, INPUT);
    pullUpDnControl(1, PUD_OFF);
    pinMode(RESET_PIN, OUTPUT);

    digitalWrite(RESET_PIN, 0);
    delay(10);
    digitalWrite(RESET_PIN, 1);

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
