#include <stdio.h>
#include <string.h>
#include <bluenrg.h>
#include <wiringPi.h>
#include <wiringPiSPI.h>
#define __USE_POSIX199309
#include <time.h>
#include <assert.h>

static const unsigned NORMAL_TIMEOUT_NS = 1000000;
static const unsigned NORMAL_LOOP_COUNT_TIMEOUT = 100000;
static const unsigned READ_LOOP_MAX_COUNTS_PER_BYTE = 10;

static const unsigned RESET_PIN = 1;
static const unsigned IRQ_PIN = 0;

typedef struct Param {
    unsigned length;
    const uint8_t *data;
} Param;

// Find out how many bytes can be read/written to/from the BlueNRG
// at the moment.
static int poll_read_write(unsigned *read, unsigned *write)
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

static void wait_for_irq()
{
    while (! digitalRead(IRQ_PIN))
        ;
}

static int wait_for_irq_with_timeout()
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

static int irq_is_high()
{
    return digitalRead(IRQ_PIN);
}

// Wait until the BlueNRG is ready to receive at least n bytes.
static int wait_to_write_n(unsigned n)
{
    unsigned write = 0;
    while (write < n) {
        int r = poll_read_write(NULL, &write);
        if (r < 0)
            return r;
    }
    return 0;
}

static int read_n_bytes(unsigned offset, uint8_t rbuf[], unsigned n)
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

static int send_command(unsigned command, const Param params[], unsigned n_params)
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

static int send_command_and_get_response(unsigned command, const Param params[], unsigned n_params, uint8_t response[], unsigned max_response_len)
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

    if (status_buf[0] != 0x04) {
         fprintf(stderr, "Unexpected packet start %u\n", status_buf[0]);
         return -1;
    }
    else {
        unsigned evt_code = status_buf[1];
        unsigned total_length = status_buf[2];
        unsigned param_length = total_length - 3;

        if (status_buf[3] != 1)
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

static int send_command_and_get_status(unsigned command, const Param params[], unsigned n_params, unsigned *status)
{
    uint8_t response[1];

    int r = send_command_and_get_response(command, params, n_params, response, 1);
    if (r < 0)
        return r;
    
    if (status)
        *status = response[0];
    
    return 0;
}

int bluenrg_init()
{
    printf("Starting...\n");

    wiringPiSetup();
    pinMode(IRQ_PIN, INPUT);
    pullUpDnControl(1, PUD_OFF);
    pinMode(RESET_PIN, OUTPUT);

    digitalWrite(RESET_PIN, 0);
    delay(10);
    digitalWrite(RESET_PIN, 1);

    wait_for_irq();

    printf("Got high IRQ following reset.\n");
    printf("Setting up SPI...\n");
    wiringPiSPISetup(0, 500000);
    printf("SPI set up.\n");

    printf("Waiting for initial IRQ high...\n");
    wait_for_irq();
    printf("IRQ is high\n");

    int r;
    unsigned read, write;

    for (;;) {
        r = poll_read_write(&read, &write);
        if (r < 0)
            return -1;

        if (read == 6) {
            break;
        }
        else if (read != 0) {
            fprintf(stderr, "Expecting 6 to read following reset, got %u\n", read);
            return -1;
        }
    }

    wait_for_irq();

    uint8_t buf1[6];
    r = read_n_bytes(0, buf1, 6);
    if (r < 0) {
        return -1;
    }
    if (buf1[0] != 0x04 || buf1[1] != 0xff || buf1[2] != 0x03 || buf1[3] != 0x01 || buf1[4] != 0x00 || buf1[5] != 0x01) {
        fprintf(stderr, "Initial response from BlueNRG unexpected: %x %x %x %x %x %x\n", buf1[0], buf1[1], buf1[2], buf1[3], buf1[4], buf1[5]);
        return -1;
    }

    printf("Initialized.\n");

    return 0;
}

int bluenrg_set_address(const uint8_t *address)
{
    const uint8_t addr_off_len[] = { 0, 6 };
    const Param addr_params[] = {
        { 2, addr_off_len },
        { 6, address }
    };
    unsigned status;
    int r = send_command_and_get_status(0xFC0C, addr_params, 2, &status);
    if (r < 0)
        return r;

    printf("Address command status: %u\n", status);
    if (status != 0) {
        fprintf(stderr, "Failed to set bluetooth address.\n");
        return -1;
    }
    printf("Address updated successfully.\n");

    return 0;
}

int bluenrg_gatt_init()
{
    printf("Initializing GATT...\n");

    unsigned status;
    int r = send_command_and_get_status(0xFD01, NULL, 0, &status);
    if (r < 0)
        return r;
    
    printf("GATT_Init command status: %u\n", status);
    if (status != 0) {
        fprintf(stderr, "Failed to init GATT.\n");
        return -1;
    }
    printf("GATT initialized successfully.\n");

    return 0;
}

// See p. 44 of DM00162667.
int bluenrg_gap_init(uint8_t role, uint8_t privacy, uint8_t device_name_char_len, bluenrg_gap_init_return *gir)
{
    printf("Initializing GAP...\n");

    uint8_t gap_init_response[7];
    gap_init_response[0] = 255;
    const uint8_t gap_role[] = { role };
    const uint8_t gap_privacy[] = { privacy };
    const uint8_t gap_name_len[] = { device_name_char_len };
    const Param gap_init_params[] = {
        { 1, gap_role },
        { 1, gap_privacy },
        { 1, gap_name_len }
    };
    int r = send_command_and_get_response(0xFC8A, gap_init_params, 3, gap_init_response, 7);
    if (r < 0)
        return r;

    printf("GAP_Init; read = %i, command status = %u\n", r, gap_init_response[0]);
    if (gap_init_response[0] != 0) {
        fprintf(stderr, "Failed to initialize GAP\n");
        return -1;
    }

    printf("GAP_Init response: ");
    for (unsigned i = 1; i < r; ++i) {
        printf("%02x ", gap_init_response[i]);
    }
    printf("\n");

    gir->service_handle = ((uint16_t)(gap_init_response[1])) + (((uint16_t)gap_init_response[2]) << 8);
    gir->dev_name_char_handle = ((uint16_t)(gap_init_response[3])) + (((uint16_t)gap_init_response[4]) << 8);
    gir->appearence_char_handle = ((uint16_t)(gap_init_response[5])) + (((uint16_t)gap_init_response[6]) << 8);

    printf("GAP initialized successfully.\n");

    return 0;
}

int bluenrg_set_device_name(const char *name, uint8_t length, const bluenrg_gap_init_return *gir)
{
    // Uses Aci_Gatt_Update_Char_Value
    // See p. 85 of DM00162667

    printf("Setting device name...\n");
    const uint8_t ucv_service_handle[] = { gir->service_handle & 0xFF, gir->service_handle >> 8 };
    const uint8_t ucv_char_handle[] = { gir->dev_name_char_handle & 0xFF, gir->dev_name_char_handle >> 8 };
    const uint8_t ucv_val_offset[] = { 0 };
    const uint8_t ucv_char_value_length[] = { length };
    const Param ucv_params[] = {
        { 2,      ucv_service_handle },
        { 2,      ucv_char_handle },
        { 1,      ucv_val_offset },
        { 1,      ucv_char_value_length },
        { length, name }
    };
    unsigned ucv_status;
    int r = send_command_and_get_status(0xFD06, ucv_params, sizeof(ucv_params)/sizeof(ucv_params[0]), &ucv_status);
    if (r < 0)
        return r;

    printf("UCV status: %u\n", ucv_status);
    if (ucv_status != 0) {
        fprintf(stderr, "Failed to set device name\n");
        return 1;
    }
    printf("Device name successfully set.\n");

    return 0;
}

int bluenrg_gatt_add_serv(uint8_t uuid_type, const uint8_t *uuid, uint8_t service_type, uint8_t max_attribute_records, uint16_t *service_handle)
{
    const uint8_t gas_service_uuid_type[] = { uuid_type };
    const uint8_t gas_service_type[] = { service_type };
    const uint8_t gas_max_attribute_records[] = { max_attribute_records };

    unsigned uuid_len = 2;
    if (uuid_type == BLUENRG_SERVICE_UUID_128BIT)
        uuid_len = 16;

    const Param gas_params[] = {
        { 1,        gas_service_uuid_type },
        { uuid_len, uuid },
        { 1,        gas_service_type },
        { 1,        gas_max_attribute_records }
    };
    uint8_t gas_response[3];
    int r  = send_command_and_get_response(0xFD02, gas_params, sizeof(gas_params)/sizeof(gas_params[0]), gas_response, 3);
    if (r < 0)
        return r;

    printf("Add service status: %u\n", gas_response[0]);
    if (gas_response[0] != 0) {
        fprintf(stderr, "Adding service failed\n");
        return -1;
    }
    *service_handle = ((uint16_t)(gas_response[1])) + (((uint16_t)(gas_response[2])) << 8);
    printf("Service was added: %04x\n", *service_handle);

    return 0;
}

int bluenrg_gatt_add_char(const bluenrg_gatt_add_char_args *args, uint16_t *char_handle)
{
    printf("Adding a service characteristic...");

    const uint8_t gac_service_handle[] = { args->service_handle & 0xFF, args->service_handle >> 8 };
    const uint8_t gac_char_value_length[] = { args->char_value_length & 0xFF, args->char_value_length >> 8 };

    unsigned uuid_length = 2;
    if (args->uuid_type == BLUENRG_SERVICE_UUID_128BIT)
        uuid_length = 16;

    const Param params[] = {
        { 2,           gac_service_handle },
        { 1,           &args->uuid_type },
        { uuid_length, args->uuid },
        { 1,           &args->char_value_length },
        { 1,           &args->char_properties },
        { 1,           &args->security_permissions },
        { 1,           &args->evt_mask },
        { 1,           &args->encryption_key_size },
        { 1,           &args->is_variable }
    };

    uint8_t response[3];
    int r = send_command_and_get_response(0xFD04, params, sizeof(params)/sizeof(params[0]), response, 3);
    if (r < 0)
        return r;

    printf("Add service charac status: %u\n", response[0]);
    if (response[0] != 0) {
        fprintf(stderr, "Failed to add service characteristic.\n");
        return -1;
    }
    printf("Successfully added service characteristic.\n");

    *char_handle = ((uint16_t)(response[0])) + (((uint16_t)(response[1])) << 8);

    return 0;
}

int bluenrg_set_auth_requirement(const bluenrg_set_auth_requirement_args *args)
{
    printf("Setting auth requirement...\n");

    const Param params[] = {
        { 1,  &args->mitm_mode },
        { 1,  &args->oob_enable },
        { 16, args->oob_data },
        { 1,  &args->min_encryption_key_size },
        { 1,  &args->max_encryption_key_size },
        { 1,  &args->use_fixed_pin },
        { 4,  args->fixed_pin },
        { 1,  &args->bonding_mode }
    };

    unsigned response;
    int r = send_command_and_get_status(0xFC86, params, sizeof(params)/sizeof(params[0]), &response);
    if (r < 0)
        return -1;

    printf("Set auth requirment status: %u\n", response);
    if (response != 0) {
        fprintf(stderr, "Failed to SAR\n");
        return -1;
    }
    printf("Auth requirement successfully set.\n");

    return 0;
}

int bluenrg_set_tx_power_level(uint8_t high_power, uint8_t pa_level)
{
    printf("Setting power level...\n");

    const Param params[] = {
        { 1, &high_power },
        { 1, &pa_level }
    };

    unsigned response;
    int r = send_command_and_get_status(0xFC0F, params, sizeof(params)/sizeof(params[0]), &response);
    if (r < 0)
        return r;

    if (response != 0) {
        fprintf(stderr, "Failed to set power level\n");
        return -1;
    }

    return 0;
}

int bluenrg_set_scan_response_data(uint8_t length, const uint8_t *data)
{
    printf("Setting scan resp data.\n");
    const uint8_t srd_data_length[] = { length };
    uint8_t srd_data[31];
    memset(srd_data, 0, 31);
    for (unsigned i = 0; i < 31 && i < length; ++i)
        srd_data[i] = data[i];
    const Param params[] = {
        { 1,  srd_data_length },
        { 31, srd_data }
    };
    unsigned status;
    int r = send_command_and_get_status(0x2009, params, sizeof(params)/sizeof(params[0]), &status);

    printf("Set scan resp data status: %u\n", status);
    if (status != 0) {
        fprintf(stderr, "Failed to set scan resp data.\n");
        return -1;
    }
    printf("Scan resp data set.\n");
}

int bluenrg_gap_set_discoverable(const bluenrg_gap_set_discoverable_args *args)
{
    printf("Setting discoverable...\n");

    uint8_t adv_interval_min[] = { args->adv_interval_min & 0xFF, args->adv_interval_min >> 8 };
    uint8_t adv_interval_max[] = { args->adv_interval_max & 0xFF, args->adv_interval_max >> 8 };
    uint8_t slave_conn_interval_min[] = { args->slave_conn_interval_min & 0xFF, args->slave_conn_interval_min >> 8 };
    uint8_t slave_conn_interval_max[] = { args->slave_conn_interval_max & 0xFF, args->slave_conn_interval_max >> 8 };

    const Param params[] = {
        { 1,                         &args->advertising_event_type },
        { 2,                         adv_interval_min },
        { 2,                         adv_interval_max },
        { 1,                         &args->address_type },
        { 1,                         &args->adv_filter_policy },
        { 1,                         &args->local_name_length },
        { args->local_name_length,   args->local_name },
        { 1,                         &args->service_uuid_length },
        { args->service_uuid_length, args->service_uuid_list },
        { 2,                         slave_conn_interval_min },
        { 2,                         slave_conn_interval_max }
    };

    unsigned status;
    int r = send_command_and_get_status(0xFC83, params, sizeof(params)/sizeof(params[0]), &status);
    if (r < 0)
        return r;

    printf("Set discoverable status: %u\n", status);
    if (status != 0) {
        fprintf(stderr, "Failed to set discoverable.\n");
        return -1;
    }

    printf("Device now discoverable.\n");

    return 0;
}

