#!/usr/bin/env python3
#
# fixed-point value approximator
#
from typing import Iterable


AVAILABLE_BITS = 15
EXPONENT = 8
DIVISOR = (1 << EXPONENT)


def bits_to_int(bits: Iterable[bool]) -> int:
    val = 0
    for bit in bits:
        val *= 2
        if bit:
            val += 1
    return val


def approximate(value: float) -> int:
    bits = AVAILABLE_BITS * [True]

    for i in range(len(bits)):
        # try setting this bit to False
        bits[i] = False
        new_value = bits_to_int(bits) / DIVISOR
        if new_value < value:
            # undershot; set it back to True
            bits[i] = True

    # finished!
    return bits_to_int(bits)


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument(dest="value", type=float)
    args = parser.parse_args()

    approximated = approximate(args.value)
    print(f"0b{approximated:b} ({approximated / DIVISOR})")


if __name__ == "__main__":
    main()
