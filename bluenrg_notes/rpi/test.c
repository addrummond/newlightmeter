#include <wiringPi.h>
#include <wiringPiSPI.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#define __USE_POSIX199309
#include <time.h>
#include <assert.h>

#define NORMAL_TIMEOUT_NS             1000000
#define NORMAL_LOOP_COUNT_TIMEOUT     100000
#define READ_LOOP_MAX_COUNTS_PER_BYTE 10

static const uint8_t SERVER_BDADDR[] = {0x12, 0x34, 0x00, 0xE1, 0x80, 0x02};

typedef struct Param {
    unsigned length;
    const uint8_t *data;
} Param;

int poll_read_write(unsigned *read, unsigned *write)
{
    uint8_t buf[5];
    buf[0] = 0x0B;
    buf[1] = 0x00;
    buf[2] = 0x00;
    buf[3] = 0x00;
    buf[4] = 0x00;

    int r = wiringPiSPIDataRW(0, buf, 5);
    if (r < 0)
        return r;
        
    if (write)
        *write = ((unsigned)(buf[1])) | (((unsigned)(buf[2])) << 8);
    if (read)
        *read = ((unsigned)(buf[3])) | (((unsigned)(buf[4])) << 8);
    
    return 0;
}

void wait_for_irq()
{
    while (! digitalRead(0))
        ;
}

int wait_for_irq_with_timeout()
{
    // Seems to be occasional bug with timeout based on CLOCK_REALTIME,
    // so we also use a loop count timeout (which should last longer).

    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) < 0)
        return -1;
    long t1n = ts.tv_nsec;

    for (unsigned i = 0; i < NORMAL_LOOP_COUNT_TIMEOUT; ++i) {
        if (digitalRead(0))
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
    return digitalRead(0);
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

    if (status_buf[0] != 0x04) {
         fprintf(stderr, "Unexpected packet start %u\n", status_buf[0]);
         return -1;
    }
    else {
        unsigned evt_code = status_buf[1];
        unsigned total_length = status_buf[2];
        unsigned param_length = status_buf[3];

        if (total_length != param_length + 3)
            return -1;

        unsigned cmd = (unsigned)(status_buf[4]) | ((unsigned)(status_buf[5]) << 8);
        if (cmd != command)
            return -1;
                    
        if (param_length > max_response_len)
            return -2;
                    
        memcpy(response, status_buf + 6, param_length);

        return param_length;
    }
}

int send_command_and_get_status(unsigned command, const Param params[], unsigned n_params, unsigned *status)
{
    uint8_t response[1];

    int r = send_command_and_get_response(command, params, n_params, response, 1);
    if (r < 0)
        return r;
    
    if (status)
        *status = response[0];
    
    return 0;
}

int main()
{
    int ec;

    printf("Starting...\n");

    wiringPiSetup();
    pinMode(0, INPUT);
    pullUpDnControl(1, PUD_OFF);
    pinMode(1, OUTPUT);

    digitalWrite(1, 0);
    delay(10);
    digitalWrite(1, 1);

    wait_for_irq();

    printf("Got high IRQ following reset.\n");
    printf("Setting up SPI...\n");
    wiringPiSPISetup(0, 500000);
    printf("SPI set up.\n");

    printf("Going through initialization sequence...\n");

    wait_for_irq();

    int r;
    unsigned read, write;

    for (;;) {
        r = poll_read_write(&read, &write);
        if (r < 0) {
            ec = 1;
            goto err;
        }
        if (read == 6)
            break;
        else if (read != 0) {
            fprintf(stderr, "Expecting 6 to read following reset, got %u\n", read);
            return 1;
        }
    }

    wait_for_irq();

    uint8_t buf1[6];
    r = read_n_bytes(0, buf1, 6);
    if (r < 0) {
        ec = 2;
        goto err;
    }
    if (buf1[0] != 0x04 || buf1[1] != 0xff || buf1[2] != 0x03 || buf1[3] != 0x01 || buf1[4] != 0x00 || buf1[5] != 0x01) {
        fprintf(stderr, "Initial response from BlueNRG unexpected: %x %x %x %x %x %x\n", buf1[0], buf1[1], buf1[2], buf1[3], buf1[4], buf1[5]);
        return 1;
    }

    printf("Initialized.\n");

    printf("Setting Bluetooth address...\n");

    const uint8_t addr_off_len[] = { 0, 6 };
    const Param addr_params[] = {
        { 2, addr_off_len },
        { 6, SERVER_BDADDR }
    };
    unsigned addr_status;
    r = send_command_and_get_status(0xFC0C, addr_params, 2, &addr_status);
    if (r < 0) {
        ec = 3;
        goto err;
    }

    printf("Address command status: %u\n", addr_status);
    if (addr_status != 0) {
        fprintf(stderr, "Failed to set bluetooth address.\n");
        return 1;
    }
    printf("Address updated successfully.\n");

    printf("Initializing GATT...\n");

    unsigned gatt_init_status;
    r = send_command_and_get_status(0xFD01, NULL, 0, &gatt_init_status);
    
    printf("GATT_Init command status: %u\n", gatt_init_status);
    if (gatt_init_status != 0) {
        fprintf(stderr, "Failed to init GATT.\n");
        return 1;
    }
    printf("GATT initialized successfully.\n");

    printf("Initializing GAP...\n");

    uint8_t gap_init_response[7];
    gap_init_response[0] = 255;
    const uint8_t gap_role[] = { 0x01/*peripheral*/ };
    const Param gap_init_params[] = {
        { 1, gap_role }
    };
    r = send_command_and_get_response(0xFC8A, gap_init_params, 1, gap_init_response, 7);

    printf("GAP_Init; read = %i, command status = %u\n", r, gap_init_response[0]);
    if (gap_init_response[0] != 0) {
        fprintf(stderr, "Failed to initialize GAP\n");
        return 1;
    }

    printf("GAP_Init response: ");
    for (unsigned i = 1; i < r; ++i) {
        printf("%02x ", gap_init_response[i]);
    }
    printf("\n");

    printf("GAP initialized successfully.\n");
    
    return 0;

err:
    printf("ERROR! %i\n", ec);
    return 1;
}
