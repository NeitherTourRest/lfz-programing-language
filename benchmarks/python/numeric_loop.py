# numeric_loop: sum of integers 1..N using an explicit while loop.
# Functionally equivalent to benchmarks/lfz/numeric_loop.lfz.
# No builtin aggregation (no sum()) -- measures loop + arithmetic.
# Usage: the size N is read from stdin (one integer line).
import sys

n = int(sys.stdin.readline())
i = 1
total = 0
while i <= n:
    total += i
    i += 1
print(f"RESULT total={total}")
