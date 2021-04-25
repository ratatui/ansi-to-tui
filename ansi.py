#!/usr/bin/env python

# for code in range(107):
#     print(f"\x1b[{code}mTHIS IS TEXT\x1b[0m")

# for z in range(255):
#     print(
#         f"\x1b[{38};{5};{z};{1};{7};{3};{5};{4}mTEXT \\x1b[{38};{5};{z}m\\1xb[0m\x1b[0m")

llist = [27, 49, 49, 49, 51, 49, 109, 49, 50, 51, 52, 27, 48, 109, ]

for l in llist:
    print(chr(l))
