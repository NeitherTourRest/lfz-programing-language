# array_builtins: range -> map -> filter -> sort -> reduce chain.
# Functionally equivalent to benchmarks/lfz/array_builtins.lfz.
# Uses the native higher-order builtins on both sides (fair comparison).
# Usage: the size N is read from stdin (one integer line).
import functools
import sys

n = int(sys.stdin.readline())
xs = range(n)
ys = map(lambda x: x * 7 % n, xs)
zs = filter(lambda x: x % 2 == 0, ys)
ss = sorted(zs)
total = functools.reduce(lambda a, b: a + b, ss, 0)
print(f"RESULT total={total}")
