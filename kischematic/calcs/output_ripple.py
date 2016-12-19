#
# Calculations of output ripple voltage for LM3670#
#

import math

max_output_current = 0.005
freq = 2000000
vout = 3.3
vin = 5.5
equiv_series_r = 0.000245
output_cap = 1e-5

input_ripple_current = (max_output_current * math.sqrt((vout/vin) * (1 - (vout/vin)))) / 2
vppc = input_ripple_current/(freq*8*output_cap)
ppesr = input_ripple_current * equiv_series_r
pprms = math.sqrt(vppc*vppc + ppesr*ppesr)

print(pprms)

