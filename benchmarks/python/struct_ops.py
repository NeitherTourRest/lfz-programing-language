# struct_ops: build a dict with N dynamic string keys, then read them back
# using sorted keys and sum the values.
# Functionally equivalent to benchmarks/lfz/struct_ops.lfz.
# Usage: the size N is read from stdin (one integer line).
import sys

n = int(sys.stdin.readline())
s = {}
i = 0
while i < n:
    s["k%d" % i] = i * i
    i += 1
total = 0
for k in sorted(s.keys()):
    total += s[k]
print(f"RESULT keys={len(s)} total={total}")
