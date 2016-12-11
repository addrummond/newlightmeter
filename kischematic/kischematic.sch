EESchema Schematic File Version 2
LIBS:power
LIBS:device
LIBS:transistors
LIBS:conn
LIBS:linear
LIBS:regul
LIBS:74xx
LIBS:cmos4000
LIBS:adc-dac
LIBS:memory
LIBS:xilinx
LIBS:microcontrollers
LIBS:dsp
LIBS:microchip
LIBS:analog_switches
LIBS:motorola
LIBS:texas
LIBS:intel
LIBS:audio
LIBS:interface
LIBS:digital-audio
LIBS:philips
LIBS:display
LIBS:cypress
LIBS:siliconi
LIBS:opto
LIBS:atmel
LIBS:contrib
LIBS:valves
LIBS:newlightmeter
LIBS:kischematic-cache
EELAYER 26 0
EELAYER END
$Descr A3 16535 11693
encoding utf-8
Sheet 1 1
Title ""
Date ""
Rev ""
Comp ""
Comment1 ""
Comment2 ""
Comment3 ""
Comment4 ""
$EndDescr
$Comp
L BlueNRG IC1
U 1 1 584AA47B
P 4750 4100
F 0 "IC1" H 5641 4153 60  0000 L CNN
F 1 "BlueNRG" H 5641 4047 60  0000 L CNN
F 2 "newlightmeter:QFN32" H 4750 4100 60  0001 C CNN
F 3 "" H 4750 4100 60  0001 C CNN
	1    4750 4100
	1    0    0    -1  
$EndComp
$Comp
L R R1
U 1 1 584AA5B8
P 2500 3950
F 0 "R1" V 2293 3950 50  0000 C CNN
F 1 "10k" V 2384 3950 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 2430 3950 50  0001 C CNN
F 3 "" H 2500 3950 50  0000 C CNN
	1    2500 3950
	0    1    1    0   
$EndComp
Text GLabel 3650 3750 0    60   Input ~ 0
SPI_MOSI
Text GLabel 3650 3850 0    60   Input ~ 0
SPI_CLK
$Comp
L GND #PWR01
U 1 1 584AA72F
P 2300 3950
F 0 "#PWR01" H 2300 3700 50  0001 C CNN
F 1 "GND" V 2305 3822 50  0000 R CNN
F 2 "" H 2300 3950 50  0000 C CNN
F 3 "" H 2300 3950 50  0000 C CNN
	1    2300 3950
	0    1    1    0   
$EndComp
$Comp
L GND #PWR02
U 1 1 584AA93D
P 3550 5000
F 0 "#PWR02" H 3550 4750 50  0001 C CNN
F 1 "GND" H 3555 4827 50  0000 C CNN
F 2 "" H 3550 5000 50  0000 C CNN
F 3 "" H 3550 5000 50  0000 C CNN
	1    3550 5000
	1    0    0    -1  
$EndComp
Text GLabel 3150 4150 0    60   Input ~ 0
VREG
$Comp
L C C2
U 1 1 584AAB34
P 4400 5700
F 0 "C2" H 4515 5746 50  0000 L CNN
F 1 "100p" H 4515 5655 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 4438 5550 50  0001 C CNN
F 3 "" H 4400 5700 50  0000 C CNN
	1    4400 5700
	1    0    0    -1  
$EndComp
$Comp
L C C5
U 1 1 584AAB64
P 5000 5700
F 0 "C5" H 5115 5746 50  0000 L CNN
F 1 "100n" H 5115 5655 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5038 5550 50  0001 C CNN
F 3 "" H 5000 5700 50  0000 C CNN
	1    5000 5700
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR03
U 1 1 584AAC13
P 4700 6150
F 0 "#PWR03" H 4700 5900 50  0001 C CNN
F 1 "GND" H 4705 5977 50  0000 C CNN
F 2 "" H 4700 6150 50  0000 C CNN
F 3 "" H 4700 6150 50  0000 C CNN
	1    4700 6150
	1    0    0    -1  
$EndComp
$Comp
L Crystal_GND23 Y2
U 1 1 584AADD9
P 6050 4950
F 0 "Y2" H 6241 4996 50  0000 L CNN
F 1 "16M" H 6241 4905 50  0000 L CNN
F 2 "newlightmeter:CRYSTAL5x3.2" H 6050 4950 50  0001 C CNN
F 3 "" H 6050 4950 50  0000 C CNN
	1    6050 4950
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR04
U 1 1 584AAEC6
P 6050 5250
F 0 "#PWR04" H 6050 5000 50  0001 C CNN
F 1 "GND" H 6055 5077 50  0000 C CNN
F 2 "" H 6050 5250 50  0000 C CNN
F 3 "" H 6050 5250 50  0000 C CNN
	1    6050 5250
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR05
U 1 1 584AAEE0
P 6050 4650
F 0 "#PWR05" H 6050 4400 50  0001 C CNN
F 1 "GND" H 6055 4477 50  0000 C CNN
F 2 "" H 6050 4650 50  0000 C CNN
F 3 "" H 6050 4650 50  0000 C CNN
	1    6050 4650
	-1   0    0    1   
$EndComp
$Comp
L C C8
U 1 1 584AAFDF
P 5850 5700
F 0 "C8" H 5965 5746 50  0000 L CNN
F 1 "15p" H 5965 5655 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5888 5550 50  0001 C CNN
F 3 "" H 5850 5700 50  0000 C CNN
	1    5850 5700
	1    0    0    -1  
$EndComp
$Comp
L C C10
U 1 1 584AB00F
P 6250 5700
F 0 "C10" H 6365 5746 50  0000 L CNN
F 1 "15p" H 6365 5655 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6288 5550 50  0001 C CNN
F 3 "" H 6250 5700 50  0000 C CNN
	1    6250 5700
	1    0    0    -1  
$EndComp
$Comp
L L L3
U 1 1 584AB152
P 6250 6200
F 0 "L3" H 6303 6246 50  0000 L CNN
F 1 "3n9" H 6303 6155 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 6250 6200 50  0001 C CNN
F 3 "" H 6250 6200 50  0000 C CNN
	1    6250 6200
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR06
U 1 1 584AB19C
P 6250 6450
F 0 "#PWR06" H 6250 6200 50  0001 C CNN
F 1 "GND" H 6255 6277 50  0000 C CNN
F 2 "" H 6250 6450 50  0000 C CNN
F 3 "" H 6250 6450 50  0000 C CNN
	1    6250 6450
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR07
U 1 1 584AB1BF
P 5850 5950
F 0 "#PWR07" H 5850 5700 50  0001 C CNN
F 1 "GND" H 5855 5777 50  0000 C CNN
F 2 "" H 5850 5950 50  0000 C CNN
F 3 "" H 5850 5950 50  0000 C CNN
	1    5850 5950
	1    0    0    -1  
$EndComp
$Comp
L Crystal Y1
U 1 1 584AB57C
P 6050 3450
F 0 "Y1" H 6050 3718 50  0000 C CNN
F 1 "32k" H 6050 3627 50  0000 C CNN
F 2 "newlightmeter:CRYSTAL3.2x1.5mm" H 6050 3450 50  0001 C CNN
F 3 "" H 6050 3450 50  0000 C CNN
	1    6050 3450
	1    0    0    -1  
$EndComp
Text GLabel 5600 2500 1    60   Input ~ 0
VREG
Text GLabel 5100 2800 1    60   Input ~ 0
BNRG_RESET
$Comp
L L L1
U 1 1 584ABF18
P 5000 1800
F 0 "L1" H 5053 1846 50  0000 L CNN
F 1 "10u" H 5053 1755 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 5000 1800 50  0001 C CNN
F 3 "" H 5000 1800 50  0000 C CNN
	1    5000 1800
	1    0    0    -1  
$EndComp
$Comp
L C C6
U 1 1 584AC1A1
P 5050 1300
F 0 "C6" H 5165 1346 50  0000 L CNN
F 1 "100n" H 5165 1255 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5088 1150 50  0001 C CNN
F 3 "" H 5050 1300 50  0000 C CNN
	1    5050 1300
	1    0    0    -1  
$EndComp
$Comp
L C C4
U 1 1 584AC257
P 4700 1300
F 0 "C4" H 4815 1346 50  0000 L CNN
F 1 "1u" H 4815 1255 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 4738 1150 50  0001 C CNN
F 3 "" H 4700 1300 50  0000 C CNN
	1    4700 1300
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR08
U 1 1 584AC470
P 4850 1000
F 0 "#PWR08" H 4850 750 50  0001 C CNN
F 1 "GND" H 4855 827 50  0000 C CNN
F 2 "" H 4850 1000 50  0000 C CNN
F 3 "" H 4850 1000 50  0000 C CNN
	1    4850 1000
	-1   0    0    1   
$EndComp
$Comp
L GND #PWR09
U 1 1 584AC773
P 4900 2800
F 0 "#PWR09" H 4900 2550 50  0001 C CNN
F 1 "GND" H 4905 2627 50  0000 C CNN
F 2 "" H 4900 2800 50  0000 C CNN
F 3 "" H 4900 2800 50  0000 C CNN
	1    4900 2800
	-1   0    0    1   
$EndComp
$Comp
L C C1
U 1 1 584AC837
P 4000 2300
F 0 "C1" H 4115 2346 50  0000 L CNN
F 1 "100p" H 4115 2255 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 4038 2150 50  0001 C CNN
F 3 "" H 4000 2300 50  0000 C CNN
	1    4000 2300
	1    0    0    -1  
$EndComp
$Comp
L C C3
U 1 1 584AC884
P 4450 2300
F 0 "C3" H 4565 2346 50  0000 L CNN
F 1 "150n" H 4565 2255 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 4488 2150 50  0001 C CNN
F 3 "" H 4450 2300 50  0000 C CNN
	1    4450 2300
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR010
U 1 1 584AC9D4
P 4300 2000
F 0 "#PWR010" H 4300 1750 50  0001 C CNN
F 1 "GND" H 4305 1827 50  0000 C CNN
F 2 "" H 4300 2000 50  0000 C CNN
F 3 "" H 4300 2000 50  0000 C CNN
	1    4300 2000
	-1   0    0    1   
$EndComp
Text GLabel 4150 3050 0    60   Input ~ 0
SPI_MISO
Text GLabel 4150 2800 0    60   Input ~ 0
BNRG_SPI_CS
$Comp
L GND #PWR011
U 1 1 584ADC83
P 3800 3250
F 0 "#PWR011" H 3800 3000 50  0001 C CNN
F 1 "GND" V 3805 3122 50  0000 R CNN
F 2 "" H 3800 3250 50  0000 C CNN
F 3 "" H 3800 3250 50  0000 C CNN
	1    3800 3250
	0    1    1    0   
$EndComp
$Comp
L C C7
U 1 1 584B1D8F
P 5850 2850
F 0 "C7" H 5965 2896 50  0000 L CNN
F 1 "22p" H 5965 2805 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5888 2700 50  0001 C CNN
F 3 "" H 5850 2850 50  0000 C CNN
	1    5850 2850
	1    0    0    -1  
$EndComp
$Comp
L C C9
U 1 1 584B1DDF
P 6250 2850
F 0 "C9" H 6365 2896 50  0000 L CNN
F 1 "22p" H 6365 2805 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6288 2700 50  0001 C CNN
F 3 "" H 6250 2850 50  0000 C CNN
	1    6250 2850
	1    0    0    -1  
$EndComp
$Comp
L L L2
U 1 1 584B20C7
P 6050 2400
F 0 "L2" H 6103 2446 50  0000 L CNN
F 1 "3n9" H 6103 2355 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 6050 2400 50  0001 C CNN
F 3 "" H 6050 2400 50  0000 C CNN
	1    6050 2400
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR012
U 1 1 584B220D
P 6050 2150
F 0 "#PWR012" H 6050 1900 50  0001 C CNN
F 1 "GND" H 6055 1977 50  0000 C CNN
F 2 "" H 6050 2150 50  0000 C CNN
F 3 "" H 6050 2150 50  0000 C CNN
	1    6050 2150
	-1   0    0    1   
$EndComp
$Comp
L C C11
U 1 1 584B6556
P 3200 4500
F 0 "C11" H 3315 4546 50  0000 L CNN
F 1 "1u" H 3315 4455 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 3238 4350 50  0001 C CNN
F 3 "" H 3200 4500 50  0000 C CNN
	1    3200 4500
	1    0    0    -1  
$EndComp
$Comp
L C C12
U 1 1 584B67DF
P 3400 4500
F 0 "C12" H 3515 4546 50  0000 L CNN
F 1 "100n" H 3515 4455 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 3438 4350 50  0001 C CNN
F 3 "" H 3400 4500 50  0000 C CNN
	1    3400 4500
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR013
U 1 1 584B6DF6
P 3300 4750
F 0 "#PWR013" H 3300 4500 50  0001 C CNN
F 1 "GND" H 3305 4577 50  0000 C CNN
F 2 "" H 3300 4750 50  0000 C CNN
F 3 "" H 3300 4750 50  0000 C CNN
	1    3300 4750
	1    0    0    -1  
$EndComp
$Comp
L C C13
U 1 1 584B7AB4
P 5300 2650
F 0 "C13" H 5415 2696 50  0000 L CNN
F 1 "1u" H 5415 2605 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5338 2500 50  0001 C CNN
F 3 "" H 5300 2650 50  0000 C CNN
	1    5300 2650
	1    0    0    -1  
$EndComp
$Comp
L C C15
U 1 1 584B7B0D
P 5500 2650
F 0 "C15" H 5615 2696 50  0000 L CNN
F 1 "100n" H 5615 2605 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5538 2500 50  0001 C CNN
F 3 "" H 5500 2650 50  0000 C CNN
	1    5500 2650
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR014
U 1 1 584B7EC7
P 5400 2400
F 0 "#PWR014" H 5400 2150 50  0001 C CNN
F 1 "GND" H 5405 2227 50  0000 C CNN
F 2 "" H 5400 2400 50  0000 C CNN
F 3 "" H 5400 2400 50  0000 C CNN
	1    5400 2400
	-1   0    0    1   
$EndComp
Text GLabel 5250 5150 3    60   Input ~ 0
VREG
$Comp
L C C14
U 1 1 584B91BA
P 5450 5400
F 0 "C14" H 5565 5446 50  0000 L CNN
F 1 "100n" H 5565 5355 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5488 5250 50  0001 C CNN
F 3 "" H 5450 5400 50  0000 C CNN
	1    5450 5400
	1    0    0    -1  
$EndComp
$Comp
L C C16
U 1 1 584B921E
P 5650 5400
F 0 "C16" H 5765 5446 50  0000 L CNN
F 1 "1u" H 5765 5355 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 5688 5250 50  0001 C CNN
F 3 "" H 5650 5400 50  0000 C CNN
	1    5650 5400
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR015
U 1 1 584B957E
P 5550 5650
F 0 "#PWR015" H 5550 5400 50  0001 C CNN
F 1 "GND" H 5555 5477 50  0000 C CNN
F 2 "" H 5550 5650 50  0000 C CNN
F 3 "" H 5550 5650 50  0000 C CNN
	1    5550 5650
	1    0    0    -1  
$EndComp
$Comp
L C C18
U 1 1 584C085D
P 6650 4050
F 0 "C18" H 6765 4096 50  0000 L CNN
F 1 "p5" H 6765 4005 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6688 3900 50  0001 C CNN
F 3 "" H 6650 4050 50  0000 C CNN
	1    6650 4050
	1    0    0    -1  
$EndComp
$Comp
L C C19
U 1 1 584C0AF1
P 6650 4750
F 0 "C19" H 6765 4796 50  0000 L CNN
F 1 "1p3" H 6765 4705 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6688 4600 50  0001 C CNN
F 3 "" H 6650 4750 50  0000 C CNN
	1    6650 4750
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR016
U 1 1 584C0CDE
P 6650 5050
F 0 "#PWR016" H 6650 4800 50  0001 C CNN
F 1 "GND" H 6655 4877 50  0000 C CNN
F 2 "" H 6650 5050 50  0000 C CNN
F 3 "" H 6650 5050 50  0000 C CNN
	1    6650 5050
	1    0    0    -1  
$EndComp
$Comp
L L L5
U 1 1 584C0EB0
P 7000 4450
F 0 "L5" H 7053 4496 50  0000 L CNN
F 1 "2n4" H 7053 4405 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 7000 4450 50  0001 C CNN
F 3 "" H 7000 4450 50  0000 C CNN
	1    7000 4450
	0    -1   -1   0   
$EndComp
$Comp
L C C20
U 1 1 584C1270
P 6950 3650
F 0 "C20" H 7065 3696 50  0000 L CNN
F 1 "1p3" H 7065 3605 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6988 3500 50  0001 C CNN
F 3 "" H 6950 3650 50  0000 C CNN
	1    6950 3650
	0    1    1    0   
$EndComp
$Comp
L L L4
U 1 1 584C1677
P 6650 3350
F 0 "L4" H 6703 3396 50  0000 L CNN
F 1 "1n3" H 6703 3305 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 6650 3350 50  0001 C CNN
F 3 "" H 6650 3350 50  0000 C CNN
	1    6650 3350
	1    0    0    -1  
$EndComp
$Comp
L C C17
U 1 1 584C1903
P 6650 2900
F 0 "C17" H 6765 2946 50  0000 L CNN
F 1 "56p" H 6765 2855 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6688 2750 50  0001 C CNN
F 3 "" H 6650 2900 50  0000 C CNN
	1    6650 2900
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR017
U 1 1 584C1BD1
P 6650 2650
F 0 "#PWR017" H 6650 2400 50  0001 C CNN
F 1 "GND" H 6655 2477 50  0000 C CNN
F 2 "" H 6650 2650 50  0000 C CNN
F 3 "" H 6650 2650 50  0000 C CNN
	1    6650 2650
	-1   0    0    1   
$EndComp
$Comp
L L L6
U 1 1 584C1EDF
P 7500 4050
F 0 "L6" H 7553 4096 50  0000 L CNN
F 1 "1n3" H 7553 4005 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 7500 4050 50  0001 C CNN
F 3 "" H 7500 4050 50  0000 C CNN
	1    7500 4050
	0    -1   -1   0   
$EndComp
$Comp
L C C21
U 1 1 584C259F
P 7850 4300
F 0 "C21" H 7965 4346 50  0000 L CNN
F 1 "p5" H 7965 4255 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 7888 4150 50  0001 C CNN
F 3 "" H 7850 4300 50  0000 C CNN
	1    7850 4300
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR018
U 1 1 584C2780
P 7850 4550
F 0 "#PWR018" H 7850 4300 50  0001 C CNN
F 1 "GND" H 7855 4377 50  0000 C CNN
F 2 "" H 7850 4550 50  0000 C CNN
F 3 "" H 7850 4550 50  0000 C CNN
	1    7850 4550
	1    0    0    -1  
$EndComp
$Comp
L C C22
U 1 1 584C2946
P 8300 4050
F 0 "C22" H 8415 4096 50  0000 L CNN
F 1 "56p" H 8415 4005 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 8338 3900 50  0001 C CNN
F 3 "" H 8300 4050 50  0000 C CNN
	1    8300 4050
	0    1    1    0   
$EndComp
$Comp
L L L7
U 1 1 584C2BED
P 8800 4050
F 0 "L7" H 8853 4096 50  0000 L CNN
F 1 "5n6" H 8853 4005 50  0000 L CNN
F 2 "Resistors_SMD:R_0603" H 8800 4050 50  0001 C CNN
F 3 "" H 8800 4050 50  0000 C CNN
	1    8800 4050
	0    -1   -1   0   
$EndComp
$Comp
L C C23
U 1 1 584C2E28
P 9250 4300
F 0 "C23" H 9365 4346 50  0000 L CNN
F 1 "p5" H 9365 4255 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 9288 4150 50  0001 C CNN
F 3 "" H 9250 4300 50  0000 C CNN
	1    9250 4300
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR019
U 1 1 584C31DE
P 9250 4550
F 0 "#PWR019" H 9250 4300 50  0001 C CNN
F 1 "GND" H 9255 4377 50  0000 C CNN
F 2 "" H 9250 4550 50  0000 C CNN
F 3 "" H 9250 4550 50  0000 C CNN
	1    9250 4550
	1    0    0    -1  
$EndComp
Text GLabel 9550 4050 2    60   Input ~ 0
ANT
$Comp
L Antenna_Dipole AE1
U 1 1 584C690D
P 8850 2150
F 0 "AE1" H 9080 2066 50  0000 L CNN
F 1 "Antenna_Dipole" H 9080 1975 50  0000 L CNN
F 2 "newlightmeter:AN2051" H 8850 2150 50  0001 C CNN
F 3 "" H 8850 2150 50  0001 C CNN
	1    8850 2150
	1    0    0    -1  
$EndComp
Text GLabel 7650 2350 0    60   Input ~ 0
ANT
$Comp
L CONN_01X01 P1
U 1 1 584C7E04
P 900 950
F 0 "P1" H 819 725 50  0000 C CNN
F 1 "CONN_01X01" H 819 816 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 950 50  0001 C CNN
F 3 "" H 900 950 50  0000 C CNN
	1    900  950 
	-1   0    0    1   
$EndComp
$Comp
L CONN_01X01 P2
U 1 1 584C869F
P 900 1300
F 0 "P2" H 819 1075 50  0000 C CNN
F 1 "CONN_01X01" H 819 1166 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 1300 50  0001 C CNN
F 3 "" H 900 1300 50  0000 C CNN
	1    900  1300
	-1   0    0    1   
$EndComp
$Comp
L CONN_01X01 P3
U 1 1 584C8723
P 900 1650
F 0 "P3" H 819 1425 50  0000 C CNN
F 1 "CONN_01X01" H 819 1516 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 1650 50  0001 C CNN
F 3 "" H 900 1650 50  0000 C CNN
	1    900  1650
	-1   0    0    1   
$EndComp
$Comp
L CONN_01X01 P4
U 1 1 584C8858
P 900 2000
F 0 "P4" H 819 1775 50  0000 C CNN
F 1 "CONN_01X01" H 819 1866 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 2000 50  0001 C CNN
F 3 "" H 900 2000 50  0000 C CNN
	1    900  2000
	-1   0    0    1   
$EndComp
Text GLabel 1350 950  2    60   Input ~ 0
VREG
Text GLabel 1350 1300 2    60   Input ~ 0
SPI_CLK
Text GLabel 1350 1650 2    60   Input ~ 0
SPI_MISO
Text GLabel 1350 2000 2    60   Input ~ 0
SPI_MOSI
$Comp
L CONN_01X01 P5
U 1 1 584CA50F
P 900 2350
F 0 "P5" H 819 2125 50  0000 C CNN
F 1 "CONN_01X01" H 819 2216 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 2350 50  0001 C CNN
F 3 "" H 900 2350 50  0000 C CNN
	1    900  2350
	-1   0    0    1   
$EndComp
Text GLabel 1350 2350 2    60   Input ~ 0
BNRG_SPI_CS
Text GLabel 2750 3550 0    60   Input ~ 0
BNRG_SPI_IRQ
$Comp
L CONN_01X01 P6
U 1 1 584CF18F
P 900 2650
F 0 "P6" H 819 2425 50  0000 C CNN
F 1 "CONN_01X01" H 819 2516 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 2650 50  0001 C CNN
F 3 "" H 900 2650 50  0000 C CNN
	1    900  2650
	-1   0    0    1   
$EndComp
Text GLabel 1350 2650 2    60   Input ~ 0
BNRG_SPI_IRQ
$Comp
L CONN_01X01 P7
U 1 1 584CF44B
P 900 3250
F 0 "P7" H 819 3025 50  0000 C CNN
F 1 "CONN_01X01" H 819 3116 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 3250 50  0001 C CNN
F 3 "" H 900 3250 50  0000 C CNN
	1    900  3250
	-1   0    0    1   
$EndComp
$Comp
L GND #PWR020
U 1 1 584CF598
P 1350 3250
F 0 "#PWR020" H 1350 3000 50  0001 C CNN
F 1 "GND" H 1355 3077 50  0000 C CNN
F 2 "" H 1350 3250 50  0000 C CNN
F 3 "" H 1350 3250 50  0000 C CNN
	1    1350 3250
	0    -1   -1   0   
$EndComp
$Comp
L CONN_01X01 P8
U 1 1 584D83B9
P 900 2950
F 0 "P8" H 819 2725 50  0000 C CNN
F 1 "CONN_01X01" H 819 2816 50  0000 C CNN
F 2 "Pin_Headers:Pin_Header_Straight_1x01" H 900 2950 50  0001 C CNN
F 3 "" H 900 2950 50  0000 C CNN
	1    900  2950
	-1   0    0    1   
$EndComp
Text GLabel 1350 2950 2    60   Input ~ 0
BNRG_RESET
$Comp
L EFM32ZG108 IC2
U 1 1 584D9005
P 10850 6700
F 0 "IC2" H 11441 6826 60  0000 L CNN
F 1 "EFM32ZG108" H 11441 6736 39  0000 L CNN
F 2 "newlightmeter:QFN24" H 10850 6700 60  0001 C CNN
F 3 "" H 10850 6700 60  0001 C CNN
	1    10850 6700
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR021
U 1 1 584D944B
P 10400 5600
F 0 "#PWR021" H 10400 5350 50  0001 C CNN
F 1 "GND" H 10405 5427 50  0000 C CNN
F 2 "" H 10400 5600 50  0000 C CNN
F 3 "" H 10400 5600 50  0000 C CNN
	1    10400 5600
	-1   0    0    1   
$EndComp
$Comp
L C C30
U 1 1 584DA6F7
P 12050 6650
F 0 "C30" H 12165 6696 50  0000 L CNN
F 1 "1u" H 12165 6605 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 12088 6500 50  0001 C CNN
F 3 "" H 12050 6650 50  0000 C CNN
	1    12050 6650
	0    1    1    0   
$EndComp
$Comp
L GND #PWR022
U 1 1 584DA984
P 12400 6650
F 0 "#PWR022" H 12400 6400 50  0001 C CNN
F 1 "GND" H 12405 6477 50  0000 C CNN
F 2 "" H 12400 6650 50  0000 C CNN
F 3 "" H 12400 6650 50  0000 C CNN
	1    12400 6650
	0    -1   -1   0   
$EndComp
$Comp
L C C31
U 1 1 584DB92F
P 12050 7000
F 0 "C31" H 12165 7046 50  0000 L CNN
F 1 "10u" H 12165 6955 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 12088 6850 50  0001 C CNN
F 3 "" H 12050 7000 50  0000 C CNN
	1    12050 7000
	0    1    1    0   
$EndComp
$Comp
L C C32
U 1 1 584DBC3B
P 12050 7400
F 0 "C32" H 12165 7446 50  0000 L CNN
F 1 "u1" H 12165 7355 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 12088 7250 50  0001 C CNN
F 3 "" H 12050 7400 50  0000 C CNN
	1    12050 7400
	0    1    1    0   
$EndComp
$Comp
L GND #PWR023
U 1 1 584DC175
P 12400 7000
F 0 "#PWR023" H 12400 6750 50  0001 C CNN
F 1 "GND" H 12405 6827 50  0000 C CNN
F 2 "" H 12400 7000 50  0000 C CNN
F 3 "" H 12400 7000 50  0000 C CNN
	1    12400 7000
	0    -1   -1   0   
$EndComp
$Comp
L C C24
U 1 1 584DC928
P 9300 6550
F 0 "C24" H 9415 6596 50  0000 L CNN
F 1 "u1" H 9415 6505 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 9338 6400 50  0001 C CNN
F 3 "" H 9300 6550 50  0000 C CNN
	1    9300 6550
	0    1    1    0   
$EndComp
$Comp
L C C25
U 1 1 584DCBFD
P 9300 6900
F 0 "C25" H 9415 6946 50  0000 L CNN
F 1 "10u" H 9415 6855 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 9338 6750 50  0001 C CNN
F 3 "" H 9300 6900 50  0000 C CNN
	1    9300 6900
	0    1    1    0   
$EndComp
$Comp
L GND #PWR024
U 1 1 584DCF82
P 8900 6550
F 0 "#PWR024" H 8900 6300 50  0001 C CNN
F 1 "GND" H 8905 6377 50  0000 C CNN
F 2 "" H 8900 6550 50  0000 C CNN
F 3 "" H 8900 6550 50  0000 C CNN
	1    8900 6550
	0    1    1    0   
$EndComp
$Comp
L C C27
U 1 1 584DDBF2
P 10800 8600
F 0 "C27" H 10915 8646 50  0000 L CNN
F 1 "10n" H 10915 8555 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 10838 8450 50  0001 C CNN
F 3 "" H 10800 8600 50  0000 C CNN
	1    10800 8600
	-1   0    0    1   
$EndComp
$Comp
L C C28
U 1 1 584DDFC0
P 11200 8600
F 0 "C28" H 11315 8646 50  0000 L CNN
F 1 "10u" H 11315 8555 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 11238 8450 50  0001 C CNN
F 3 "" H 11200 8600 50  0000 C CNN
	1    11200 8600
	-1   0    0    1   
$EndComp
$Comp
L GND #PWR025
U 1 1 584DE579
P 11200 8950
F 0 "#PWR025" H 11200 8700 50  0001 C CNN
F 1 "GND" H 11205 8777 50  0000 C CNN
F 2 "" H 11200 8950 50  0000 C CNN
F 3 "" H 11200 8950 50  0000 C CNN
	1    11200 8950
	1    0    0    -1  
$EndComp
$Comp
L C C29
U 1 1 584DE9F2
P 11700 7800
F 0 "C29" H 11815 7846 50  0000 L CNN
F 1 "10n" H 11815 7755 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 11738 7650 50  0001 C CNN
F 3 "" H 11700 7800 50  0000 C CNN
	1    11700 7800
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR026
U 1 1 584DECE1
P 11700 8150
F 0 "#PWR026" H 11700 7900 50  0001 C CNN
F 1 "GND" H 11705 7977 50  0000 C CNN
F 2 "" H 11700 8150 50  0000 C CNN
F 3 "" H 11700 8150 50  0000 C CNN
	1    11700 8150
	1    0    0    -1  
$EndComp
Text GLabel 11000 5750 1    60   Input ~ 0
SWDIO
Text GLabel 11100 5750 1    60   Input ~ 0
SWCLK
Text GLabel 11700 6850 3    60   Input ~ 0
I2C_SCL
Text GLabel 11500 6950 3    60   Input ~ 0
I2C_SDA
Text GLabel 10000 7400 0    60   Input ~ 0
BNRG_SPI_CS
Text GLabel 9300 7200 0    60   Input ~ 0
SPI_CLK
Text GLabel 10200 6650 0    60   Input ~ 0
SPI_MOSI
Text GLabel 10200 6750 0    60   Input ~ 0
SPI_MISO
$Comp
L C C26
U 1 1 584F43E0
P 10800 5600
F 0 "C26" H 10915 5646 50  0000 L CNN
F 1 "u1" H 10915 5555 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 10838 5450 50  0001 C CNN
F 3 "" H 10800 5600 50  0000 C CNN
	1    10800 5600
	-1   0    0    1   
$EndComp
$Comp
L GND #PWR027
U 1 1 584F466A
P 10800 5300
F 0 "#PWR027" H 10800 5050 50  0001 C CNN
F 1 "GND" H 10805 5127 50  0000 C CNN
F 2 "" H 10800 5300 50  0000 C CNN
F 3 "" H 10800 5300 50  0000 C CNN
	1    10800 5300
	-1   0    0    1   
$EndComp
Text GLabel 10050 6200 0    60   Input ~ 0
BNRG_SPI_IRQ
$Comp
L R R2
U 1 1 584FC55B
P 11900 3600
F 0 "R2" V 11693 3600 50  0000 C CNN
F 1 "4k7" V 11784 3600 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 11830 3600 50  0001 C CNN
F 3 "" H 11900 3600 50  0000 C CNN
	1    11900 3600
	0    1    1    0   
$EndComp
$Comp
L R R3
U 1 1 584FC7DB
P 11900 4000
F 0 "R3" V 11693 4000 50  0000 C CNN
F 1 "4k7" V 11784 4000 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 11830 4000 50  0001 C CNN
F 3 "" H 11900 4000 50  0000 C CNN
	1    11900 4000
	0    1    1    0   
$EndComp
Text GLabel 12300 4000 2    60   Input ~ 0
VREG
Text GLabel 12300 3600 2    60   Input ~ 0
VREG
Text GLabel 11600 3600 0    60   Input ~ 0
I2C_SDA
Text GLabel 11600 4000 0    60   Input ~ 0
I2C_SCL
$Comp
L AAT3672IWO-4.2-1 IC4
U 1 1 58509BD9
P 5900 8650
F 0 "IC4" H 5875 9287 60  0000 C CNN
F 1 "AAT3672IWO-4.2-1" H 5875 9181 60  0000 C CNN
F 2 "newlightmeter:AAT3672IWO" H 5950 7750 60  0001 C CNN
F 3 "" H 5950 7750 60  0001 C CNN
	1    5900 8650
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR028
U 1 1 5850A1DE
P 5900 9500
F 0 "#PWR028" H 5900 9250 50  0001 C CNN
F 1 "GND" H 5905 9327 50  0000 C CNN
F 2 "" H 5900 9500 50  0000 C CNN
F 3 "" H 5900 9500 50  0000 C CNN
	1    5900 9500
	1    0    0    -1  
$EndComp
$Comp
L C C35
U 1 1 5850A4B0
P 6600 9100
F 0 "C35" H 6715 9146 50  0000 L CNN
F 1 "u1" H 6715 9055 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 6638 8950 50  0001 C CNN
F 3 "" H 6600 9100 50  0000 C CNN
	1    6600 9100
	-1   0    0    1   
$EndComp
Text GLabel 6650 8400 2    60   Input ~ 0
VBATSAFE
Text GLabel 6650 8300 2    60   Input ~ 0
VLOAD
$Comp
L R R8
U 1 1 5850C191
P 7450 8500
F 0 "R8" V 7243 8500 50  0000 C CNN
F 1 "10k" V 7334 8500 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 7380 8500 50  0001 C CNN
F 3 "" H 7450 8500 50  0000 C CNN
	1    7450 8500
	0    1    1    0   
$EndComp
Text GLabel 7700 8500 2    60   Input ~ 0
VUSB
$Comp
L Thermistor_NTC TH1
U 1 1 5850D0C5
P 7450 8900
F 0 "TH1" V 7160 8900 50  0000 C CNN
F 1 "10k" V 7251 8900 50  0000 C CNN
F 2 "Resistors_SMD:R_0603" H 7450 8950 50  0001 C CNN
F 3 "" H 7450 8950 50  0001 C CNN
	1    7450 8900
	0    1    1    0   
$EndComp
$Comp
L GND #PWR029
U 1 1 5850D43D
P 7700 8900
F 0 "#PWR029" H 7700 8650 50  0001 C CNN
F 1 "GND" H 7705 8727 50  0000 C CNN
F 2 "" H 7700 8900 50  0000 C CNN
F 3 "" H 7700 8900 50  0000 C CNN
	1    7700 8900
	0    -1   -1   0   
$EndComp
Text GLabel 4200 8500 0    60   Input ~ 0
VUSB
$Comp
L C C34
U 1 1 5850FD75
P 4350 8850
F 0 "C34" H 4465 8896 50  0000 L CNN
F 1 "1u" H 4465 8805 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 4388 8700 50  0001 C CNN
F 3 "" H 4350 8850 50  0000 C CNN
	1    4350 8850
	-1   0    0    1   
$EndComp
$Comp
L GND #PWR030
U 1 1 5850FF5F
P 4350 9100
F 0 "#PWR030" H 4350 8850 50  0001 C CNN
F 1 "GND" H 4355 8927 50  0000 C CNN
F 2 "" H 4350 9100 50  0000 C CNN
F 3 "" H 4350 9100 50  0000 C CNN
	1    4350 9100
	1    0    0    -1  
$EndComp
Text GLabel 5000 8800 0    60   Input ~ 0
VBATSAFE
$Comp
L R R6
U 1 1 58510F31
P 5100 9450
F 0 "R6" V 4893 9450 50  0000 C CNN
F 1 "2M7%" V 4984 9450 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 5030 9450 50  0001 C CNN
F 3 "" H 5100 9450 50  0000 C CNN
	1    5100 9450
	-1   0    0    1   
$EndComp
$Comp
L R R7
U 1 1 5851113E
P 5450 9450
F 0 "R7" V 5243 9450 50  0000 C CNN
F 1 "1M6%" V 5334 9450 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 5380 9450 50  0001 C CNN
F 3 "" H 5450 9450 50  0000 C CNN
	1    5450 9450
	-1   0    0    1   
$EndComp
$Comp
L GND #PWR031
U 1 1 58511672
P 5250 9850
F 0 "#PWR031" H 5250 9600 50  0001 C CNN
F 1 "GND" H 5255 9677 50  0000 C CNN
F 2 "" H 5250 9850 50  0000 C CNN
F 3 "" H 5250 9850 50  0000 C CNN
	1    5250 9850
	1    0    0    -1  
$EndComp
Text GLabel 5100 8300 0    60   Input ~ 0
PSTAT1
Text GLabel 5100 8400 0    60   Input ~ 0
PSTAT2
$Comp
L AP9101C IC3
U 1 1 585201C9
P 2200 8900
F 0 "IC3" H 2200 9537 60  0000 C CNN
F 1 "AP9101C" H 2200 9431 60  0000 C CNN
F 2 "newlightmeter:SOT25" H 2200 8900 60  0001 C CNN
F 3 "" H 2200 8900 60  0001 C CNN
	1    2200 8900
	1    0    0    -1  
$EndComp
$Comp
L R R4
U 1 1 58520EE2
P 1250 8550
F 0 "R4" V 1043 8550 50  0000 C CNN
F 1 "390" V 1134 8550 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 1180 8550 50  0001 C CNN
F 3 "" H 1250 8550 50  0000 C CNN
	1    1250 8550
	0    -1   -1   0   
$EndComp
$Comp
L C C33
U 1 1 5852164E
P 1500 9150
F 0 "C33" H 1615 9196 50  0000 L CNN
F 1 "100n" H 1615 9105 50  0000 L CNN
F 2 "Capacitors_SMD:C_0402" H 1538 9000 50  0001 C CNN
F 3 "" H 1500 9150 50  0000 C CNN
	1    1500 9150
	-1   0    0    1   
$EndComp
Text GLabel 900  8950 0    60   Input ~ 0
BAT+
Text GLabel 900  9100 0    60   Input ~ 0
BAT-
Wire Wire Line
	3900 3750 3650 3750
Wire Wire Line
	3900 3850 3650 3850
Wire Wire Line
	2650 3950 3900 3950
Wire Wire Line
	2350 3950 2300 3950
Wire Wire Line
	3900 4050 3550 4050
Wire Wire Line
	3550 4050 3550 5000
Wire Wire Line
	3900 4250 3650 4250
Wire Wire Line
	3650 4250 3650 4750
Wire Wire Line
	3900 4350 3750 4350
Wire Wire Line
	3750 4350 3750 4750
Wire Wire Line
	3900 4450 3850 4450
Wire Wire Line
	3850 4450 3850 5150
Wire Wire Line
	3850 4750 3550 4750
Connection ~ 3550 4750
Wire Wire Line
	3150 4150 3900 4150
Wire Wire Line
	3850 5150 4400 5150
Connection ~ 3850 4750
Wire Wire Line
	4600 5150 4600 5350
Wire Wire Line
	4600 5350 4200 5350
Wire Wire Line
	4200 5350 4200 5150
Connection ~ 4200 5150
Wire Wire Line
	4500 5150 4500 5250
Wire Wire Line
	4500 5250 4200 5250
Connection ~ 4200 5250
Wire Wire Line
	4700 5150 4700 5550
Wire Wire Line
	4400 5550 5000 5550
Connection ~ 4700 5550
Wire Wire Line
	5000 6000 5000 5850
Wire Wire Line
	4400 6000 5000 6000
Wire Wire Line
	4400 6000 4400 5850
Wire Wire Line
	4700 6000 4700 6150
Connection ~ 4700 6000
Wire Wire Line
	5600 4450 5850 4450
Wire Wire Line
	5850 4450 5850 5550
Wire Wire Line
	5600 4350 6250 4350
Wire Wire Line
	6250 4350 6250 5550
Wire Wire Line
	5850 4950 5900 4950
Wire Wire Line
	6250 4950 6200 4950
Wire Wire Line
	6050 4750 6050 4650
Wire Wire Line
	6050 5150 6050 5250
Connection ~ 5850 4950
Wire Wire Line
	6250 5850 6250 6050
Wire Wire Line
	5850 5950 5850 5850
Wire Wire Line
	6250 6450 6250 6350
Wire Wire Line
	6250 3950 5600 3950
Wire Wire Line
	5850 3850 5600 3850
Wire Wire Line
	5850 3000 5850 3850
Wire Wire Line
	6250 3000 6250 3950
Wire Wire Line
	5850 3450 5900 3450
Wire Wire Line
	6250 3450 6200 3450
Wire Wire Line
	5600 2500 5600 3750
Wire Wire Line
	5100 3050 5100 2800
Wire Wire Line
	5000 1650 5000 1500
Wire Wire Line
	4700 1500 4800 1500
Wire Wire Line
	5050 1500 5050 1450
Wire Wire Line
	4700 1150 5050 1150
Wire Wire Line
	4850 1150 4850 1000
Connection ~ 4850 1150
Wire Wire Line
	4700 1450 4700 1500
Wire Wire Line
	4800 1500 4800 3050
Connection ~ 4800 1500
Wire Wire Line
	5000 1500 5050 1500
Wire Wire Line
	4900 2800 4900 3050
Wire Wire Line
	4700 3050 4700 2550
Wire Wire Line
	4700 2550 4450 2550
Wire Wire Line
	4450 2550 4450 2450
Wire Wire Line
	4450 2450 4000 2450
Connection ~ 4450 2450
Wire Wire Line
	4000 2150 4450 2150
Wire Wire Line
	4300 2150 4300 2000
Connection ~ 4300 2150
Wire Wire Line
	4400 3050 4150 3050
Wire Wire Line
	4500 3050 4500 2800
Wire Wire Line
	4500 2800 4150 2800
Wire Wire Line
	4600 3050 4600 2950
Wire Wire Line
	4600 2950 4900 2950
Connection ~ 4900 2950
Wire Wire Line
	3900 3250 3800 3250
Wire Wire Line
	6350 3650 6800 3650
Wire Wire Line
	6350 3650 6350 4050
Wire Wire Line
	6350 4050 5600 4050
Wire Wire Line
	6350 4450 6850 4450
Wire Wire Line
	6350 4150 6350 4450
Wire Wire Line
	6350 4150 5600 4150
Connection ~ 6250 3450
Connection ~ 5850 3450
Wire Wire Line
	5850 2700 6250 2700
Wire Wire Line
	6050 2700 6050 2550
Connection ~ 6050 2700
Wire Wire Line
	6050 2250 6050 2150
Connection ~ 6250 4950
Wire Wire Line
	5000 1950 5000 3050
Wire Wire Line
	3200 4150 3200 4350
Connection ~ 3200 4150
Wire Wire Line
	3400 4150 3400 4350
Connection ~ 3400 4150
Wire Wire Line
	3200 4650 3400 4650
Wire Wire Line
	3300 4650 3300 4750
Connection ~ 3300 4650
Wire Wire Line
	5300 2800 5600 2800
Connection ~ 5600 2800
Connection ~ 5500 2800
Wire Wire Line
	5300 2500 5500 2500
Wire Wire Line
	5400 2500 5400 2400
Connection ~ 5400 2500
Wire Wire Line
	5600 4250 5650 4250
Wire Wire Line
	5650 4250 5650 5250
Wire Wire Line
	5250 5150 5650 5150
Wire Wire Line
	5450 5150 5450 5250
Connection ~ 5450 5150
Connection ~ 5650 5150
Wire Wire Line
	5450 5550 5650 5550
Wire Wire Line
	5550 5550 5550 5650
Connection ~ 5550 5550
Wire Wire Line
	6650 3500 6650 3900
Wire Wire Line
	6650 4200 6650 4600
Connection ~ 6650 4450
Wire Wire Line
	6650 4900 6650 5050
Wire Wire Line
	7250 4450 7150 4450
Wire Wire Line
	7250 3650 7250 4450
Wire Wire Line
	7250 3650 7100 3650
Connection ~ 6650 3650
Wire Wire Line
	6650 3200 6650 3050
Wire Wire Line
	6650 2750 6650 2650
Wire Wire Line
	7250 4050 7350 4050
Connection ~ 7250 4050
Wire Wire Line
	7650 4050 8150 4050
Wire Wire Line
	7850 4050 7850 4150
Wire Wire Line
	7850 4450 7850 4550
Connection ~ 7850 4050
Wire Wire Line
	8450 4050 8650 4050
Wire Wire Line
	8950 4050 9550 4050
Wire Wire Line
	9250 4050 9250 4150
Wire Wire Line
	9250 4450 9250 4550
Connection ~ 9250 4050
Wire Wire Line
	7650 2350 8850 2350
Wire Wire Line
	1100 950  1350 950 
Wire Wire Line
	1100 1300 1350 1300
Wire Wire Line
	1100 1650 1350 1650
Wire Wire Line
	1100 2000 1350 2000
Wire Wire Line
	1100 2350 1350 2350
Wire Wire Line
	2750 3950 2750 3550
Connection ~ 2750 3950
Wire Wire Line
	1100 2650 1350 2650
Wire Wire Line
	1100 3250 1350 3250
Wire Wire Line
	1100 2950 1350 2950
Wire Wire Line
	10400 5900 10400 5600
Wire Wire Line
	11400 6650 11900 6650
Wire Wire Line
	12200 6650 12400 6650
Wire Wire Line
	11400 6750 11850 6750
Wire Wire Line
	12200 7000 12400 7000
Wire Wire Line
	12250 7000 12250 7400
Wire Wire Line
	12250 7400 12200 7400
Connection ~ 12250 7000
Wire Wire Line
	9450 6550 10300 6550
Wire Wire Line
	9150 6550 9150 6900
Wire Wire Line
	9450 6900 9600 6900
Wire Wire Line
	9600 6900 9600 6550
Connection ~ 9600 6550
Wire Wire Line
	9150 6550 8900 6550
Wire Wire Line
	11850 6750 11850 7400
Wire Wire Line
	11850 7000 11900 7000
Wire Wire Line
	11850 7400 11900 7400
Connection ~ 11850 7000
Wire Wire Line
	10800 7500 10800 8450
Wire Wire Line
	11200 8450 11200 8400
Wire Wire Line
	11200 8400 10800 8400
Connection ~ 10800 8400
Wire Wire Line
	10800 8750 10800 8850
Wire Wire Line
	10800 8850 11200 8850
Wire Wire Line
	11200 8750 11200 8950
Connection ~ 11200 8850
Wire Wire Line
	11100 7500 11100 7550
Wire Wire Line
	11100 7550 11700 7550
Wire Wire Line
	11700 7550 11700 7650
Wire Wire Line
	11700 7950 11700 8150
Wire Wire Line
	11000 5900 11000 5750
Wire Wire Line
	11100 5900 11100 5750
Wire Wire Line
	11400 6850 11700 6850
Wire Wire Line
	11400 6950 11500 6950
Wire Wire Line
	10300 6950 10300 7400
Wire Wire Line
	10300 7400 10000 7400
Wire Wire Line
	10300 6850 10200 6850
Wire Wire Line
	10200 6850 10200 7200
Wire Wire Line
	10200 7200 9300 7200
Wire Wire Line
	10300 6650 10200 6650
Wire Wire Line
	10300 6750 10200 6750
Wire Wire Line
	10800 5900 10800 5750
Wire Wire Line
	10800 5450 10800 5300
Wire Wire Line
	10300 6450 10050 6450
Wire Wire Line
	10050 6450 10050 6200
Wire Wire Line
	12050 3600 12300 3600
Wire Wire Line
	12050 4000 12300 4000
Wire Wire Line
	11750 3600 11600 3600
Wire Wire Line
	11750 4000 11600 4000
Wire Wire Line
	5900 9350 5900 9500
Wire Wire Line
	6400 8700 6600 8700
Wire Wire Line
	6600 8700 6600 8950
Wire Wire Line
	6600 9250 6600 9400
Wire Wire Line
	6600 9400 5900 9400
Connection ~ 5900 9400
Wire Wire Line
	6400 8400 6650 8400
Wire Wire Line
	6400 8300 6650 8300
Wire Wire Line
	6400 8500 7300 8500
Wire Wire Line
	7600 8500 7700 8500
Wire Wire Line
	7200 8500 7200 8900
Connection ~ 7200 8500
Wire Wire Line
	7200 8900 7300 8900
Wire Wire Line
	7600 8900 7700 8900
Wire Wire Line
	4200 8500 5350 8500
Wire Wire Line
	5200 8500 5200 8600
Wire Wire Line
	5200 8600 5350 8600
Connection ~ 5200 8500
Wire Wire Line
	5050 8500 5050 8700
Wire Wire Line
	5050 8700 5350 8700
Connection ~ 5050 8500
Wire Wire Line
	4350 8500 4350 8700
Connection ~ 4350 8500
Wire Wire Line
	4350 9000 4350 9100
Wire Wire Line
	5350 8800 5000 8800
Wire Wire Line
	5350 8900 5100 8900
Wire Wire Line
	5100 8900 5100 9300
Wire Wire Line
	5350 9000 5300 9000
Wire Wire Line
	5300 9000 5300 9300
Wire Wire Line
	5300 9300 5450 9300
Wire Wire Line
	5100 9600 5100 9700
Wire Wire Line
	5100 9700 5450 9700
Wire Wire Line
	5450 9700 5450 9600
Wire Wire Line
	5250 9700 5250 9850
Connection ~ 5250 9700
Wire Wire Line
	5350 8300 5100 8300
Wire Wire Line
	5350 8400 5100 8400
Wire Wire Line
	1400 8550 1750 8550
Wire Wire Line
	1100 8550 1000 8550
Wire Wire Line
	1500 8550 1500 9000
Connection ~ 1500 8550
Wire Wire Line
	1500 9300 1500 9550
Wire Wire Line
	1000 9550 1700 9550
Wire Wire Line
	2850 9550 3150 9550
Wire Wire Line
	1000 8100 1000 8950
Wire Wire Line
	1000 8950 900  8950
Wire Wire Line
	1000 9100 900  9100
Wire Wire Line
	1750 8750 1700 8750
Wire Wire Line
	1700 8750 1700 9800
Connection ~ 1500 9550
Connection ~ 1000 9550
Wire Wire Line
	2650 8650 2850 8650
Wire Wire Line
	2850 8650 2850 9100
$Comp
L R R5
U 1 1 585233FC
P 2850 9250
F 0 "R5" V 2643 9250 50  0000 C CNN
F 1 "2k7" V 2734 9250 50  0000 C CNN
F 2 "Resistors_SMD:R_0402" V 2780 9250 50  0001 C CNN
F 3 "" H 2850 9250 50  0000 C CNN
	1    2850 9250
	-1   0    0    1   
$EndComp
Wire Wire Line
	2850 9400 2850 9800
Connection ~ 1700 9550
Connection ~ 2850 9550
Wire Wire Line
	1000 9100 1000 9550
$Comp
L GND #PWR032
U 1 1 58523FD0
P 3150 9550
F 0 "#PWR032" H 3150 9300 50  0001 C CNN
F 1 "GND" H 3155 9377 50  0000 C CNN
F 2 "" H 3150 9550 50  0000 C CNN
F 3 "" H 3150 9550 50  0000 C CNN
	1    3150 9550
	0    -1   -1   0   
$EndComp
Wire Wire Line
	1000 8100 3150 8100
Connection ~ 1000 8550
Text GLabel 3150 8100 2    60   Input ~ 0
VBATSAFE
Wire Wire Line
	2250 9800 2350 9800
Wire Wire Line
	1700 9800 1850 9800
Wire Wire Line
	2400 10100 2600 10100
Wire Wire Line
	2400 10100 2400 9250
Wire Wire Line
	2400 9250 2300 9250
Wire Wire Line
	2300 9250 2300 9200
Wire Wire Line
	2100 9200 2050 9200
Wire Wire Line
	2050 9200 2050 9500
Wire Wire Line
	2850 9800 2750 9800
Wire Wire Line
	2050 9500 2000 9500
$Comp
L NCHANNELMOSFET Q1
U 1 1 584DBD89
P 2050 9700
F 0 "Q1" V 2286 9700 50  0000 C CNN
F 1 "NCHANNELMOSFET" V 2377 9700 50  0000 C CNN
F 2 "TO_SOT_Packages_SMD:SOT-23" V 2468 9700 50  0000 C CIN
F 3 "" H 2050 9700 50  0000 L CNN
	1    2050 9700
	0    1    1    0   
$EndComp
$Comp
L NCHANNELMOSFET Q2
U 1 1 584DBEC7
P 2550 9900
F 0 "Q2" V 2968 9900 50  0000 C CNN
F 1 "NCHANNELMOSFET" V 2877 9900 50  0000 C CNN
F 2 "TO_SOT_Packages_SMD:SOT-23" V 2786 9900 50  0000 C CIN
F 3 "" H 2550 9900 50  0000 L CNN
	1    2550 9900
	0    -1   -1   0   
$EndComp
$EndSCHEMATC
