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

#define PR(x) { sizeof(x)/sizeof(uint8_t), x }

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
    const uint8_t gap_privacy[] = { 0x00 };
    const uint8_t gap_name_len[] = { 0x10 };
    const Param gap_init_params[] = {
        { 1, gap_role },
        // These parameters are new for -MS, documented on p. 46 of http://www.st.com/content/ccc/resource/technical/document/programming_manual/1c/7e/d3/a6/d0/52/4a/35/DM00141271.pdf/files/DM00141271.pdf/jcr:content/translations/en.DM00141271.pdf
        { 1, gap_privacy },
        { 1, gap_name_len }
    };
    r = send_command_and_get_response(0xFC8A, gap_init_params, 3, gap_init_response, 7);

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

    unsigned service_handle = ((unsigned)(gap_init_response[1])) + (((unsigned)gap_init_response[2]) << 8);
    unsigned dev_name_char_handle = ((unsigned)(gap_init_response[3])) + (((unsigned)gap_init_response[4]) << 8);

    printf("GAP initialized successfully.\n");

    printf("Setting device name...\n");
    const uint8_t ucv_service_handle[] = { service_handle & 0xFF, service_handle >> 8 };
    const uint8_t ucv_char_handle[] = { dev_name_char_handle & 0xFF, dev_name_char_handle >> 8 };
    const uint8_t ucv_val_offset[] = { 0 };
    const uint8_t ucv_char_value_length[] = { 0x10 };
    const uint8_t ucv_char_value[] = "Vaquita porpoise";
    const Param ucv_params[] = {
        { 2,  ucv_service_handle },
        { 2,  ucv_char_handle },
        { 1,  ucv_val_offset },
        { 1,  ucv_char_value_length },
        { 16, ucv_char_value }
    };
    unsigned ucv_status;
    r = send_command_and_get_status(0xFD06, ucv_params, sizeof(ucv_params)/sizeof(ucv_params[0]), &ucv_status);

    printf("UCV status: %u\n", ucv_status);
    if (ucv_status != 0) {
        fprintf(stderr, "Failed to set device name\n");
        return 1;
    }
    printf("Device name successfully set.\n");

    printf("Adding a GATT service...\n");
    const uint8_t gas_service_uuid_type[] = { 0x01 };
    const uint8_t gas_service_uuid[] = { 0x01, 0xA0 };
    const uint8_t gas_service_type[] = { 0x01 };
    const uint8_t gas_max_attribute_records[] = { 0x06 };
    const Param gas_params[] = {
        { 1, gas_service_uuid_type },
        { 2, gas_service_uuid },
        { 1, gas_service_type },
        { 1, gas_max_attribute_records }
    };
    uint8_t gas_response[3];
    r = send_command_and_get_response(0xFD02, gas_params, sizeof(gas_params)/sizeof(gas_params[0]), gas_response, 3);

    printf("Add service status: %u\n", gas_response[0]);
    if (gas_response[0] != 0) {
        fprintf(stderr, "Adding service failed\n");
        return 1;
    }
    unsigned service_u = ((unsigned)(gas_response[1])) + (((unsigned)(gas_response[2])) << 8);
    printf("Service was added: %04x\n", service_u);

    // See p. 24 of http://www.st.com/content/ccc/resource/technical/document/user_manual/11/7b/ae/96/a8/b9/48/bf/DM00099259.pdf/files/DM00099259.pdf/jcr:content/translations/en.DM00099259.pdf
    printf("Adding a service characteristic...\n");
    const uint8_t gac_service_handle[] = { service_u & 0xFF, service_u >> 8 };
    const uint8_t gac_char_uuid_type[] = { 0x01 };
    const uint8_t gac_char_uuid[] = { 0x01, 0xA0 };
    const uint8_t gac_char_value_length[] = { 10 };
    const uint8_t gac_char_properties[] = { 0x1A };
    const uint8_t gac_security_permissions[] = { 0x00 };
    const uint8_t gac_event_mask[] = { 0x01 };
    const uint8_t gac_encryption_key_size[] = { 0x07 };
    const uint8_t gac_is_variable[] = { 0x01 };
    const Param gac_params[] = {
        PR(gac_service_handle),
        PR(gac_char_uuid_type),
        PR(gac_char_uuid),
        PR(gac_char_value_length),
        PR(gac_char_properties),
        PR(gac_security_permissions),
        PR(gac_event_mask),
        PR(gac_encryption_key_size),
        PR(gac_is_variable)
    };
    uint8_t gac_response[3];
    r = send_command_and_get_response(0xFD04, gac_params, sizeof(gac_params)/sizeof(gac_params[0]), gac_response, 3);

    printf("Add service charac status: %u\n", gac_response[0]);
    if (gac_response[0] != 0) {
        fprintf(stderr, "Failed to add service characteristic.\n");
        return 1;
    }
    printf("Successfully added service characteristic.\n");

    unsigned service_charac_handle = ((unsigned)(gac_response[0])) + (((unsigned)(gac_response[1])) << 8);

    printf("Setting auth requirement...\n");

    const uint8_t sar_mimt_mode[] = { 0x00 };
    const uint8_t sar_oob_enable[] = { 0x00 };
    const uint8_t sar_oob_data[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
    const uint8_t sar_min_encryption_key_size[] = { 0x07 };
    const uint8_t sar_max_encryption_key_size[] = { 0x10 };
    const uint8_t sar_use_fixed_pin[] = { 0x01 };
    const uint8_t sar_fixed_pin[] = { 123, 0x00, 0x00, 0x00 };
    const uint8_t sar_bonding_mode[] = { 0x01 };
    const Param sar_params[] = {
        { 1,  sar_mimt_mode },
        { 1,  sar_oob_enable },
        { 16, sar_oob_data },
        { 1,  sar_min_encryption_key_size },
        { 1,  sar_max_encryption_key_size },
        { 1,  sar_use_fixed_pin },
        { 4,  sar_fixed_pin },
        { 1,  sar_bonding_mode }
    };
    unsigned sar_response;
    r = send_command_and_get_status(0xFC86, sar_params, sizeof(sar_params)/sizeof(sar_params[0]), &sar_response);

    printf("Set auth requirment status: %u\n", sar_response);
    if (sar_response != 0) {
        fprintf(stderr, "Failed to SAR\n");
        return 1;
    }
    printf("Auth requirement successfully set.\n");

    printf("Setting power level...\n");

    const uint8_t spl_en_high_power[] = { 0x00 };
    const uint8_t spl_pa_level[] = { 0x04 };
    const Param spl_params[] = {
        { 1, spl_en_high_power },
        { 1, spl_pa_level }
    };
    unsigned spl_response;
    r = send_command_and_get_status(0xFC0F, spl_params, sizeof(spl_params)/sizeof(spl_params[0]), &spl_response);

    printf("Set power level status: %u\n", spl_response);
    if (sar_response != 0) {
        fprintf(stderr, "Failed to set power level\n");
        return 1;
    }
    printf("Power level successfully set.\n");

    // p. 1061
    // Set_advertising_parameters
    // Set_advertising_data
    // Set_scan_resp_data
    // Set_advertise_enable
#if 0
    printf("Setting advertizing parameters...\n");
    const uint8_t sap_min_inverval[] = { 0x00, 0x08 };
    const uint8_t sap_max_interval[] = { 0x00, 0x08 };
    const uint8_t sap_advtype[] = { 0x00 };
    const uint8_t sap_own_bdaddr_type[] = { 0x01 };
    const uint8_t sap_direct_bdaddr_type[] = { 0x01 };
    const uint8_t sap_direct_bdaddr[] = { 0, 0, 0, 0, 0, 0 };
    const uint8_t sap_channel_map[] = { 0b00000111 };
    const uint8_t sap_filter_policy[] = { 0x00 };
    const Param sap_params[] = {
        { 2, sap_min_inverval },
        { 2, sap_max_interval },
        { 1, sap_advtype },
        { 1, sap_own_bdaddr_type },
        { 1, sap_direct_bdaddr_type },
        { 6, sap_direct_bdaddr },
        { 1, sap_channel_map },
        { 1, sap_filter_policy }
    };
    unsigned sap_status;
    r = send_command_and_get_status(0x2006, sap_params, sizeof(sap_params)/sizeof(sap_params[0]), &sap_status);

    printf("Set advertizing parameters status: %u\n", sap_status);
    if (sap_status != 0) {
        fprintf(stderr, "Failed to set advertizing parameters.\n");
        return 1;
    }
    printf("Advertizing parameters set.\n");

    printf("Setting advertizing data...\n");
    const uint8_t sad_data_length[] = { 0x00 };
    uint8_t sad_data[31];
    for (unsigned i = 0; i < 31; ++i)
        sad_data[i] = 0;
    const Param sad_params[] = {
        { 1,  sad_data_length },
        { 31, sad_data }
    };
    unsigned sad_status;
    r = send_command_and_get_status(0x2008, sad_params, sizeof(sad_params)/sizeof(sad_params[0]), &sad_status);

    printf("Set advertizing data status: %u\n", sad_status);
    if (sad_status != 0) {
        fprintf(stderr, "Failed to set advertizing data.\n");
        return 1;
    }
    printf("Advertizing data set.\n");
#endif

    printf("Setting scan resp data.\n");
    const uint8_t srd_data_length[] = { 0x00 };
    uint8_t srd_data[31];
    for (unsigned i = 0; i < 31; ++i)
        srd_data[i] = 0;
    const Param srd_params[] = {
        { 1,  srd_data_length },
        { 31, srd_data }
    };
    unsigned srd_status;
    r = send_command_and_get_status(0x2009, srd_params, sizeof(srd_params)/sizeof(srd_params[0]), &srd_status);

    printf("Set scan resp data status: %u\n", srd_status);
    if (srd_status != 0) {
        fprintf(stderr, "Failed to set scan resp data.\n");
        return 1;
    }
    printf("Scan resp data set.\n");

    printf("Setting discoverable...\n");
    //const unsigned adv_interval_min = (800*1000)/625;
    //const unsigned adv_interval_max = (900*1000)/625;
    //const unsigned conn_interval_min = (100*1000)/1250;
    //const unsigned conn_interval_max = (300*1000)/1250;
    const uint8_t gsd_adv_event_type[] = { 0x00 };
    const uint8_t gsd_adv_interval_min[] = { 0x00, 0x08 }; //{ adv_interval_min & 0xFF, adv_interval_min >> 8 };
    const uint8_t gsd_adv_interval_max[] = { 0x00, 0x09 }; //{ adv_interval_max & 0xFF, adv_interval_max >> 8 };
    const uint8_t gsd_address_type[] = { 0x00 };//{ 0x01 };
    const uint8_t gsd_adv_filter_policy[] = { 0x00 };
    const uint8_t gsd_local_name[] = "Vaquita Porpoise";
    const uint8_t gsd_local_name_length[] = { sizeof(gsd_local_name)/sizeof(uint8_t) };
    const uint8_t gsd_service_uuid_length[] = { 0x00 };
    //const uint8_t gsd_service_uuid_list[] = { service_u & 0xFF, service_u >> 8 };
    const uint8_t gsd_slave_conn_interval_min[] = { 0x00, 0x00 }; //{ conn_interval_min & 0xFF, conn_interval_min >> 8 };
    const uint8_t gsd_slave_conn_interval_max[] = { 0x00, 0x00 }; //{ conn_interval_max & 0xFF, conn_interval_max >> 8 };
    const Param gsd_params[] = {
        { 1,  gsd_adv_event_type }, 
        { 2,  gsd_adv_interval_min },
        { 2,  gsd_adv_interval_max },
        { 1,  gsd_address_type},
        { 1,  gsd_adv_filter_policy },
        { 1,  gsd_local_name_length },
        { 16, gsd_local_name },
        { 1,  gsd_service_uuid_length },
        //{ 2,  gsd_service_uuid_list },
        { 2,  gsd_slave_conn_interval_min },
        { 2,  gsd_slave_conn_interval_max }
    };
    unsigned gsd_status;
    r = send_command_and_get_status(0xFC83, gsd_params, sizeof(gsd_params)/sizeof(gsd_params[0]), &gsd_status);

    printf("Set discoverable status: %u\n", gsd_status);
    if (gsd_status != 0) {
        fprintf(stderr, "Fail to set discoverable.\n");
        return 1;
    }
    printf("Device now discoverable.\n");

#if 0
    printf("Setting advertizing enable...\n");
    const uint8_t sae_en[] = { 0x01 };
    const Param sae_params[] = {
        { 1, sae_en }
    };
    unsigned sae_status;
    r = send_command_and_get_status(0x200A, sae_params, 1, &sae_status);

    printf("Set adv enable status: %u\n", sae_status);
    if (sae_status != 0) {
        fprintf(stderr, "Failed to enable adv.\n");
        return 1;
    }
    printf("Adv enabled.\n");
#endif

    return 0;

err:
    printf("ERROR! %i\n", ec);
    return 1;
}
