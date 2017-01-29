Some random notes on the BlueNRG and its SPI interface.

Start of a command packet signaled with 01.
Start of an event packet signaled with 04.

Some evidence that "04 0e 04 01 0c fc 00" is a normal response upon setting
device address:

    https://www.google.co.uk/webhp?sourceid=chrome-instant&rlz=1C5CHFA_enGB693GB693&ion=1&espv=2&ie=UTF-8#q=bluenrg+%2204+0e+04+01+0c+fc+00%22

Potentially useful info on correct sequence of startup commands (although
this is presumably being posted because it doesn't quite work):

http://bbs.csdn.net/topics/391021421

