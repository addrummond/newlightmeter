material a drf=0.0 srf=0.0 at=0.0 c1=1.0
material p drf=0.75 srf=0.75 rff=0.75 at=0.000001 c1=0.01 c2=0.1

line a/a -40 40 -40 -40
line a/a 40 40 40 -40
arc p/a (20) 0 0 0 20 0 -20
colbeam n=10 i=1.0 l=1.0 |- -10 -20 -10 20