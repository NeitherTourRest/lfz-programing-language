# string_ops: build N strings with interpolation, trim them, join, replace,
# split, repeat. Functionally equivalent to benchmarks/lfz/string_ops.lfz.
# Usage: the size N is read from stdin (one integer line).
import sys

n = int(sys.stdin.readline())
parts = []
i = 0
while i < n:
    parts.append("  item%d  " % i)
    i += 1
cleaned = list(map(lambda s: s.strip(), parts))
joined = ",".join(cleaned)
renamed = joined.replace("item", "ITEM")
words = renamed.split(",")
tag = "ab" * 3
print(f"RESULT chars={len(joined)} words={len(words)} tag={tag}")
