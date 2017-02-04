#include <stdint.h>
#include <stdbool.h>
#include <em_chip.h>
#include <em_gpio.h>
#include <em_cmu.h>
#include <em_device.h>
#include <em_i2c.h>
#include <em_emu.h>
#include <rtt.h>

#define I2C_PORT gpioPortD
#define I2C_SDA_PIN 6
#define I2C_SCL_PIN 7

#define I2C_ADDR        0b0101001
#define ACCEL_I2C_ADDR  0x0D

#define DATA0LOW  0xC
#define DATA0HIGH 0xD
#define DATA1LOW  0xE
#define DATA2HIGH 0xF

// static volatile uint32_t ms_ticks;

//void SysTick_Handler()
//{
//    ++ms_ticks;
//}

//void delay(uint32_t delayTicks)
//{
//    uint32_t cur_ticks = ms_ticks;
//    while ((ms_ticks - cur_ticks) < delayTicks);
//}

static void i2c_init()
{
    SEGGER_RTT_printf(0, "Initializing...\n");
    GPIO_PinModeSet(I2C_PORT, I2C_SCL_PIN, gpioModeWiredAndFilter, 1); // configure SCL pin as open drain output
    GPIO_PinModeSet(I2C_PORT, I2C_SDA_PIN, gpioModeWiredAndFilter, 1); // configure SDA pin as open drain output
    
    I2C0->ROUTE = I2C_ROUTE_SDAPEN | I2C_ROUTE_SCLPEN | (1 << _I2C_ROUTE_LOCATION_SHIFT);
    I2C0->CTRL = I2C_CTRL_AUTOACK | I2C_CTRL_AUTOSN;

    for (unsigned i = 0; i < 9; i++)
    {
        /*
         * TBD: Seems to be clocking at appr 80kHz-120kHz depending on compiler
         * optimization when running at 14MHz. A bit high for standard mode devices,
         * but DVK only has fast mode devices. Need however to add some time
         * measurement in order to not be dependable on frequency and code executed.
        */
        GPIO_PinModeSet(I2C_PORT, I2C_SCL_PIN, gpioModeWiredAndFilter, 0);
        GPIO_PinModeSet(I2C_PORT, I2C_SCL_PIN, gpioModeWiredAndFilter, 1);
    }

    I2C_Init_TypeDef i2c_init = I2C_INIT_DEFAULT;/*{
        .enable = true,
        .master = true,
        .refFreq = 0,
        .freq = I2C_FREQ_STANDARD_MAX,
        .clhr = i2cClockHLRAsymetric
    };*/

    SEGGER_RTT_printf(0, "Initializing I2C...\n");
    I2C_Init(I2C0, &i2c_init);
    SEGGER_RTT_printf(0, "I2C initialized.\n");

    NVIC_ClearPendingIRQ(I2C0_IRQn);
    NVIC_EnableIRQ(I2C0_IRQn);
}

static void print_stat(int status)
{
    if (status == 0)
        SEGGER_RTT_printf(0, "S: I2C done.\n");
    else if (status == -1)
        SEGGER_RTT_printf(0, "S: NACK\n");
    else if (status == -2)
        SEGGER_RTT_printf(0, "S: BUS ERR\n");
    else if (status == -3)
        SEGGER_RTT_printf(0, "S: ARB LOST\n");
    else if (status == -4)
        SEGGER_RTT_printf(0, "S: USAGE FAULT\n");
    else if (status == -5)
        SEGGER_RTT_printf(0, "S: SW FAULT\n");
}

static void accel_write_reg(uint8_t reg, uint8_t val)
{
    uint8_t wbuf[] = { reg, val };
    I2C_TransferSeq_TypeDef i2c_transfer = {
        .addr = ACCEL_I2C_ADDR,
        .flags = I2C_FLAG_WRITE,
        .buf[0].data = wbuf,
        .buf[0].len = sizeof(wbuf)/sizeof(wbuf[0])
    };
    SEGGER_RTT_printf(0, "Starting transfer..\n");
    int status = I2C_TransferInit(I2C0, &i2c_transfer);
    while (status == i2cTransferInProgress)
        status = I2C_Transfer(I2C0);
    SEGGER_RTT_printf(0, "Ending transfer..\n");
    print_stat(status);
}

static void i2c_test1_setup()
{
    SEGGER_RTT_printf(0, "Init accel...\n");
    accel_write_reg(0x2A, 0b10000001);
    accel_write_reg(0x2B, 0);
    accel_write_reg(0x2C, 0);
    accel_write_reg(0x2D, 0);
    SEGGER_RTT_printf(0, "Done init.\n");    
}

static void i2c_test1_read()
{
    SEGGER_RTT_printf(0, "Trying to read accel reg.\n");    
    uint8_t wbuf[] = { 0x05 };
    uint8_t rbuf[1];
    I2C_TransferSeq_TypeDef i2c_transfer = {
        .addr = ACCEL_I2C_ADDR,
        .flags = I2C_FLAG_WRITE_READ,
        .buf[0].data = wbuf,
        .buf[0].len = sizeof(wbuf)/sizeof(wbuf[0]),
        .buf[1].data = rbuf,
        .buf[1].len = sizeof(rbuf)/sizeof(rbuf[0])
    };
    SEGGER_RTT_printf(0, "Accel reg read %u\n", rbuf[0]);    
    int status = I2C_TransferInit(I2C0, &i2c_transfer);
    while (status == i2cTransferInProgress)
        status = I2C_Transfer(I2C0);
}

static void i2c_test2()
{
    uint8_t wbuf[] = { 0b11000000 | DATA0LOW };
    uint8_t rbuf[2];
    I2C_TransferSeq_TypeDef i2c_transfer = {
        .addr = I2C_ADDR,
        .flags = I2C_FLAG_WRITE_READ,
        .buf[0].data = wbuf,
        .buf[0].len = sizeof(wbuf)/sizeof(wbuf[0]),
        .buf[1].data = rbuf,
        .buf[1].len = sizeof(rbuf)/sizeof(rbuf[0])
    };
    SEGGER_RTT_printf(0, "Before\n");    
    int status = I2C_TransferInit(I2C0, &i2c_transfer);
    SEGGER_RTT_printf(0, "PreeLoop\n");
    while (status == i2cTransferInProgress); {
        SEGGER_RTT_printf(0, "Loop\n");
        status = I2C_Transfer(I2C0);
    }
    
    SEGGER_RTT_printf(0, "Initial transfer complete.\n");
    SEGGER_RTT_printf(0, "Read complete %u %u\n", rbuf[0], rbuf[1]);
}

int main()
{
    CHIP_Init();

    CMU_ClockEnable(cmuClock_HFPER, true);
    CMU_ClockEnable(cmuClock_I2C0, true);
    CMU_ClockEnable(cmuClock_GPIO, true);

    rtt_init();
    SEGGER_RTT_printf(0, "\n\nHello RTT console; core clock freq = %u.\n", CMU_ClockFreqGet(cmuClock_CORE));

    //GPIO_PinModeSet(I2C_PORT, I2C_SCL_PIN, gpioModeWiredAndFilter, 1); // configure SCL pin as open drain output
    //GPIO_PinModeSet(I2C_PORT, I2C_SDA_PIN, gpioModeWiredAndFilter, 1); // configure SDA pin as open drain output

    i2c_init();
    i2c_test1_setup();
    for (;;) {
       i2c_test1_read();
    }

    // Set SysTick timer for 1 msec interrupts.
    //if (SysTick_Config(CMU_ClockFreqGet(cmuClock_CORE) / 1000))
    //    while (1); // Error.

    for (;;)
       ;

    return 0;
}
