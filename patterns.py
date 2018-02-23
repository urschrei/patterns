#!/usr/bin/env python
# coding: utf-8

"""
patterns.py
author: Stephan Hügel
"""
from collections import Counter


def generate_patterns(haystack):
    """ Generate tuples of integer patterns from ASCII uppercase strings """
    total = 0
    stack = [0] * 128
    pattern = []

    for char in haystack:
        byte = ord(char)
        needle = stack[byte]
        if needle == 0:
            total += 1
            stack[byte] = total
            needle = total
        pattern.append(needle - 1)
    # we need tuples because lists aren't hashable
    return tuple(pattern)


if __name__ == "__main__":
    with open("words.txt", 'r') as f:
        counts = Counter((generate_patterns(line) for line in f))
        friendly = sum(
            {key: counts[key] for key in counts if counts[key] > 1}.values()
        )
    print("Number of friendly strings: %s" % friendly)
