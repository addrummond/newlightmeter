#include <wiringPi.h>
#include <wiringPiSPI.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

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
    buf[0] = 0x0B;
    for (unsigned i = 1; i < n + 5; ++i)
        buf[i] = 0;
    
    int r = wiringPiSPIDataRW(0, buf, n + 5);
    if (r < 0)
        return r;
    
    for (unsigned i = 0; i < n; ++i) {
        rbuf[i] = buf[i + 5];
    }

    for (unsigned i = 0; i < n + 5; ++i) {
        printf("%x ", buf[i]);
    }
    printf("\n");

    return 0;
}

int send_command(unsigned command, const Param params[], unsigned n_params)
{
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

int send_command_and_get_status(unsigned command, const Param params[], unsigned n_params, unsigned *status)
{
    int r = send_command(command, params, n_params);
    if (r < 0)
        return r;
    
    wait_for_irq();
    
    unsigned total_read = 0;
    uint8_t status_buf[7];

    while (irq_is_high()) {
        unsigned read;
        r = poll_read_write(&read, NULL);
        if (r < 0)
            return r;
                
        if (read > 0) {
            unsigned n = 7 - total_read;

            if (read > n)
                return -1;

            r = read_n_bytes(total_read, status_buf, n);
            if (r < 0)
                return r;
            
            total_read += n;

            if (total_read == 7) {
                if (status_buf[0] == 0) {
                    printf("IGNORING DUMMY INFO %x %x %x %x\n", status_buf[0], status_buf[1], status_buf[2], status_buf[3]);
                    total_read = 0;
                }
                else if (status_buf[0] == 0x04 && status_buf[1] != 0x0e && status_buf[2] != 0x04 && status_buf[3] != 0x01) {
                    return -1;
                }
                else {
                    unsigned cmd = (unsigned)(status_buf[4]) | ((unsigned)(status_buf[5]) << 8);
                    if (cmd != command)
                        return -1;
                
                    if (status)
                        *status = status_buf[6];
                
                    return 0;
                }
            }
        }
    }
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

    printf("Got 6\n");

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
    
    return 0;

err:
    printf("ERROR! %i\n", ec);
    return 1;
}
