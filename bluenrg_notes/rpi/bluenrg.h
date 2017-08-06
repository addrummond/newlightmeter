#ifndef BLUENRG_H
#define BLUENRG_H

#include <stdint.h>

int bluenrg_init(void);
int bluenrg_set_address(const uint8_t *address);
int bluenrg_gatt_init(void);

static const uint8_t BLUENRG_GAP_ROLE_PERIPHERAL    = 0x01;
static const uint8_t BLUENRG_GAP_ROLE_BROADCASTER   = 0x02;
static const uint8_t BLUENRG_GAP_ROLE_CENTRAL       = 0x03;
static const uint8_t BLUENRG_GAP_ROLE_OBSERVER      = 0x04;

static const uint8_t BLUENRG_GAP_PRIVACY_ON = 1;
static const uint8_t BLUENRG_GAP_PRIVACY_OFF = 0;

typedef struct bluenrg_gap_init_return_ {
    uint16_t service_handle;
    uint16_t dev_name_char_handle;
    uint16_t appearence_char_handle;
} bluenrg_gap_init_return;

int bluenrg_gap_init(uint8_t role, uint8_t privacy, uint8_t device_name_char_len, bluenrg_gap_init_return *gir);

int bluenrg_set_device_name(const char *name, uint8_t length, const bluenrg_gap_init_return *gir);

static const uint8_t BLUENRG_SERVICE_UUID_16BIT  = 0x01;
static const uint8_t BLUENRG_SERVICE_UUID_128BIT = 0x02;

static const uint8_t BLUENRG_SERVICE_PRIMARY     = 0x01;
static const uint8_t BLUENRG_SERVICE_SECONDARY   = 0x02;

int bluenrg_gatt_add_serv(uint8_t uuid_type, const uint8_t *uuid, uint8_t service_type, uint8_t max_attribute_records, uint16_t *service_handle);

static const uint8_t BLUENRG_GATT_SECURITY_PERM_NONE               = 0x00;
static const uint8_t BLUENRG_GATT_SECURITY_PERM_AUTH_TO_READ       = (0x01 | 0x02);
static const uint8_t BLUENRG_GATT_SECURITY_PERM_ENCRYPT_TO_READ    = 0x04;
static const uint8_t BLUENRG_GATT_SECURITY_PERM_AUTH_TO_WRITE      = (0x08 | 0x10);
static const uint8_t BLUENRG_GATT_SECURITY_PERM_ENCRYPT_TO_WRITE   = 0x20;

static const uint8_t BLUENRG_GATT_SERVER_ATTR_WRITE                = 0x01;
static const uint8_t BLUENRG_GATT_INTIMATE_AND_WAIT_FOR_APPL_AUTH  = 0x02;
static const uint8_t BLUENRG_GATT_INTIMATE_APPL_WHEN_READ_N_WAIT   = 0x04;

typedef struct bluenrg_gatt_add_char_args_ {
    uint16_t service_handle;
    uint8_t uuid_type;
    uint8_t *uuid;
    uint8_t char_value_length; // Note that this is listed as a two-byte field in DM00162667, but with a tricksy footnote!
    uint8_t char_properties;
    uint8_t security_permissions;
    uint8_t evt_mask;
    uint8_t encryption_key_size;
    uint8_t is_variable;
} bluenrg_gatt_add_char_args;

static const uint8_t BLUENRG_GATT_CHR_PROP_BROADCAST           = 0x01;
static const uint8_t BLUENRG_GATT_CHR_PROP_READ                = 0x02;
static const uint8_t BLUENRG_GATT_CHR_PROP_WRITE_WITHOUT_RESP  = 0x04;
static const uint8_t BLUENRG_GATT_CHR_PROP_WRITE               = 0x08;
static const uint8_t BLUENRG_GATT_CHR_PROP_NOTIFY              = 0x10;
static const uint8_t BLUENRG_GATT_CHR_PROP_INDICATE            = 0x20;
static const uint8_t BLUENRG_GATT_CHR_PROP_AUTH                = 0x40;
static const uint8_t BLUENRG_GATT_CHR_PROP_EXT_PROP            = 0x80;

int bluenrg_gatt_add_char(const bluenrg_gatt_add_char_args *args, uint16_t *char_handle);

static const uint8_t BLUENRG_MITM_REQUIRED        = 0x01;
static const uint8_t BLUENRG_MITM_NOT_REQUIRED    = 0x00;
static const uint8_t BLUENRG_OOB_ENABLED          = 0x01;
static const uint8_t BLUENRG_OOB_DISABLED         = 0x00;
static const uint8_t BLUENRG_USE_FIXED_PIN        = 0x01;
static const uint8_t BLUENRG_DO_NOT_USE_FIXED_PIN = 0x00;
static const uint8_t BLUENRG_BONDING_REQUIRED     = 0x01;
static const uint8_t BLUENRG_BONDING_NOT_REQUIRED = 0x00;

typedef struct bluenrg_set_auth_requirement_args_ {
    uint8_t mitm_mode;
    uint8_t oob_enable;
    uint8_t oob_data[16];
    uint8_t min_encryption_key_size;
    uint8_t max_encryption_key_size;
    uint8_t use_fixed_pin;
    uint8_t fixed_pin[4];
    uint8_t bonding_mode;
} bluenrg_set_auth_requirement_args;

int bluenrg_set_auth_requirement(const bluenrg_set_auth_requirement_args *args);

static const uint8_t BLUENRG_ENABLE_HIGH_POWER  = 0x01;
static const uint8_t BLUENRG_DISABLE_HIGH_POWER = 0x00;

int bluenrg_set_tx_power_level(uint8_t high_power, uint8_t pa_level);
int bluenrg_set_scan_response_data(uint8_t length, const uint8_t *data);

static const uint8_t BLUENRG_CONNECTABLE_UNDIRECTED_ADVERTISING     = 0x00;
static const uint8_t BLUENRG_SCANNABLE_UNDIRECTED_ADVERTISING       = 0x02;
static const uint8_t BLUENRG_NON_CONNECTABLE_UNDIRECTED_ADVERTISING = 0x03;

static const uint8_t BLUENRG_ADVERTISING_ALLOW_ANY_ANY     = 0x00;
static const uint8_t BLUENRG_ADVERTISING_ALLOW_WHITE_ANY   = 0x01;
static const uint8_t BLUENRG_ADVERTISING_ALLOW_ANY_WHITE   = 0x02;
static const uint8_t BLUENRG_ADVERTISING_ALLOW_WHITE_WHITE = 0x0;

static const uint8_t BLUENRG_PUBLIC_DEVICE_ADDRESS = 0x00;
static const uint8_t BLUENRG_RANDOM_DEVICE_ADDRESS = 0x01;

typedef struct bluenrg_gap_set_discoverable_args_ {
    uint8_t advertising_event_type;
    uint16_t adv_interval_min;
    uint16_t adv_interval_max;
    uint8_t address_type;
    uint8_t adv_filter_policy;
    uint8_t local_name_length;
    char *local_name;
    uint8_t service_uuid_length;
    uint8_t *service_uuid_list;
    uint16_t slave_conn_interval_min;
    uint16_t slave_conn_interval_max;
} bluenrg_gap_set_discoverable_args;

int bluenrg_gap_set_discoverable(const bluenrg_gap_set_discoverable_args *args);

#endif
