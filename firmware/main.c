#include <stdint.h>
#include <em_chip.h>
#include <em_gpio.h>
#include <em_cmu.h>
#include <em_device.h>
#include <rtt.h>

static volatile uint32_t ms_ticks;

//void SysTick_Handler()
//{
//    ++ms_ticks;
//}

//void delay(uint32_t delayTicks)
//{
//    uint32_t cur_ticks = ms_ticks;
//    while ((ms_ticks - cur_ticks) < delayTicks);
//}

int main()
{
    CHIP_Init();

    CMU_ClockEnable(cmuClock_HFPER, true);
    CMU_ClockEnable(cmuClock_GPIO, true);

    // Set SysTick timer for 1 msec interrupts.
    //if (SysTick_Config(CMU_ClockFreqGet(cmuClock_CORE) / 1000))
    //    while (1); // Error.

    rtt_init();
    SEGGER_RTT_printf(0, "\n\nHello RTT console; core clock freq = %u.\n", CMU_ClockFreqGet(cmuClock_CORE));

    for (;;)
       ;

    return 0;
}
