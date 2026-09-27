# recursion_fib: naive (exponential) recursive Fibonacci fib(n).
# Functionally equivalent to benchmarks/lfz/recursion_fib.lfz.
# Measures recursive-call overhead. N is small because work ~ fib(n).
# Usage: the size N is read from stdin (one integer line).
import sys


def fib(k):
    if k < 2:
        return k
    return fib(k - 1) + fib(k - 2)


n = int(sys.stdin.readline())
print(f"RESULT fib={fib(n)}")
