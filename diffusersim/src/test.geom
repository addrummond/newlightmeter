material a drf=0.0 srf=0.0 at=0.0 c1=1.0
material p drf=0.75 srf=0.75 rff=0.75 at=0.000001 c1=0.01 c2=0.1

#line p/a -20 20 20 20
#line p/a 20 20 20 -20
#line p/a 20 -20 -20 -20
#line p/a -20 -20 -20 20

#line a/p -40 -40 -40 40
#line a/p 40 40 40 -40

line a/p -20 -20 -20 20
line a/p 20 -20 20 20

line a/p -40 -40 -40 40
line p/a 40 40 40 -40

line a/p 50 50 50 -50

arc a/p (10) 0 0 0 10 10 0

colbeam n=20 i=1.0 l=1.0 |- -30 -30 -25 30
#colbeam n=1 i=1.0 l=1.0 |- -30 29 -29.5 30