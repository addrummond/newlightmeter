#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <bluenrg.h>

static const uint8_t SERVER_BDADDR[] = {0x12, 0x34, 0x00, 0xE1, 0x80, 0x02};

static const char DEVICE_NAME[] = "Alex's BlueNRG";
static const size_t DEVICE_NAME_LEN = ((sizeof(DEVICE_NAME)/sizeof(char))-1);

int main()
{
    if (bluenrg_init() < 0)
        return 1;
    if (bluenrg_set_address(SERVER_BDADDR) < 0)
        return 1;
    if (bluenrg_gatt_init() < 0)
        return 1;

    bluenrg_gap_init_return gir;
    int r = bluenrg_gap_init(
        BLUENRG_GAP_ROLE_PERIPHERAL,
        BLUENRG_GAP_PRIVACY_OFF,
        DEVICE_NAME_LEN,
        &gir
    );
    if (r < 0)
        return 1;

    if (bluenrg_set_device_name(DEVICE_NAME, DEVICE_NAME_LEN, &gir) < 0)
        return 1;

    const uint8_t gas_service_uuid[] = { 0x01, 0xA0 };
    uint16_t service_handle;
    r = bluenrg_gatt_add_serv(
        BLUENRG_SERVICE_UUID_16BIT,
        gas_service_uuid,
        BLUENRG_SERVICE_PRIMARY,
        0x06,
        &service_handle
    );
    if (r < 0)
        return 1;

    uint8_t my_uuid[] = { 0x01, 0xA0 };
    bluenrg_gatt_add_char_args gaca;
    gaca.service_handle = service_handle;
    gaca.uuid_type = BLUENRG_SERVICE_UUID_16BIT;
    gaca.uuid = my_uuid;
    gaca.char_value_length = 10;
    gaca.security_permissions = BLUENRG_GATT_SECURITY_PERM_NONE;
    gaca.char_properties = BLUENRG_GATT_CHR_PROP_READ   |
                           BLUENRG_GATT_CHR_PROP_WRITE  |
                           BLUENRG_GATT_CHR_PROP_NOTIFY;
    gaca.evt_mask = BLUENRG_GATT_SERVER_ATTR_WRITE;
    gaca.encryption_key_size = 0x07;
    gaca.is_variable = 0x01;
    uint16_t char_handle;
    r = bluenrg_gatt_add_char(&gaca, &char_handle);
    if (r < 0)
        return 1;

    bluenrg_set_auth_requirement_args sara;
    sara.mitm_mode = BLUENRG_MITM_NOT_REQUIRED;
    sara.oob_enable = BLUENRG_OOB_DISABLED;
    memset(&sara.oob_data, 0, sizeof(sara.oob_data));
    sara.min_encryption_key_size = 0x07;
    sara.max_encryption_key_size = 0x10;
    sara.use_fixed_pin = BLUENRG_USE_FIXED_PIN;
    memset(&sara.fixed_pin, 0, sizeof(sara.fixed_pin));
    sara.fixed_pin[0] = 123;
    sara.bonding_mode = BLUENRG_BONDING_REQUIRED;
    r = bluenrg_set_auth_requirement(&sara);
    if (r < 0)
        return 1;

    if (bluenrg_set_tx_power_level(BLUENRG_ENABLE_HIGH_POWER, 0x07) < 0)
        return 1;

    if (bluenrg_set_scan_response_data(0, NULL) < 0)
        return 1;

    bluenrg_gap_set_discoverable_args gsda;
    gsda.advertising_event_type = BLUENRG_CONNECTABLE_UNDIRECTED_ADVERTISING;
    gsda.adv_interval_min = 0;
    gsda.adv_interval_max = 0;
    gsda.address_type = BLUENRG_PUBLIC_DEVICE_ADDRESS; 
    gsda.adv_filter_policy = BLUENRG_ADVERTISING_ALLOW_ANY_ANY;
    gsda.local_name_length = 14;
    gsda.local_name = "Alex's BlueNRG";
    gsda.service_uuid_length = 0;
    gsda.service_uuid_list = NULL;
    gsda.slave_conn_interval_min = (100*1000)/1250;
    gsda.slave_conn_interval_max = (300*1000)/1250;
    r = bluenrg_gap_set_discoverable(&gsda);
    if (r < 0)
        return 1;

    return 0;
}

//
// Below: Some code that turned out not to be necessary to make the device discoverable.
//

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
