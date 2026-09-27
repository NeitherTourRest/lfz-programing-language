# function_calls: N high-frequency calls to a small pure function.
# Functionally equivalent to benchmarks/lfz/function_calls.lfz.
# Isolates per-call overhead (no recursion, no builtins).
# Usage: the size N is read from stdin (one integer line).
import sys


def step(x):
    return (x * 31 + 7) % 1000003


n = int(sys.stdin.readline())
acc = 1
i = 0
while i < n:
    acc = step(acc)
    i += 1
print(f"RESULT acc={acc}")
