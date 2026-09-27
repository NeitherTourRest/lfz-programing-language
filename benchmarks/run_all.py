#!/usr/bin/env python3
"""One-click LFZ vs Python benchmark harness (standard library only).

Measures, for each benchmark and each input size:
  * LFZ   : `lfz run benchmarks/lfz/<name>.lfz`
  * Python: `python benchmarks/python/<name>.py`
with the size N piped to stdin (one integer line).

Methodology
-----------
* release build of the interpreter is required.
* WARMUP untimed runs first (default 2), then RUNS timed runs (default 5).
* wall-clock is measured with time.perf_counter around the whole subprocess
  (process startup + load + execute). This is reported as the "total" figure.
* a noop pair (fixtures/noop.*) is measured the same way to quantify a startup
  baseline; the "net" figure subtracts that baseline.
* median / min / max are reported; the median is the headline number.
* LFZ and Python stdout must match exactly on every run, otherwise that
  (benchmark, size) cell is marked INVALID and excluded.

Reproduce
---------
    cargo build --release
    python benchmarks/run_all.py

Options: --warmup N --runs N --quick  (quick = first size of each benchmark only)
Raw per-run data is written to benchmarks/results/raw.json.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BENCH_DIR = ROOT / "benchmarks"
LFZ_BIN = ROOT / "target" / "release" / ("lfz.exe" if os.name == "nt" else "lfz")

# name -> (description, [sizes])
BENCHMARKS: dict[str, tuple[str, list[int]]] = {
    "numeric_loop": ("while-loop sum 1..N (no builtins)", [10_000, 100_000, 1_000_000]),
    "function_calls": ("N calls to a small pure function", [10_000, 100_000, 1_000_000]),
    "recursion_fib": ("naive recursive fib(n)", [20, 24, 28]),
    "array_builtins": ("range|>map|>filter|>sort|>reduce", [1_000, 10_000, 100_000]),
    "struct_ops": ("struct/dict: N writes + keys() + sum", [5_000, 15_000, 40_000]),
    "string_ops": ("build/trim/join/replace/split strings", [3_000, 10_000, 30_000]),
}


def run_once(cmd: list[str], size: int) -> tuple[float, str, str, int]:
    """Run one command with `size` on stdin; return (seconds, stdout, stderr, rc)."""
    t0 = time.perf_counter()
    proc = subprocess.run(
        cmd,
        input=f"{size}\n",
        capture_output=True,
        text=True,
        cwd=str(ROOT),
    )
    dt = time.perf_counter() - t0
    return dt, proc.stdout, proc.stderr, proc.returncode


def lfz_cmd(name: str) -> list[str]:
    return [str(LFZ_BIN), "run", str(BENCH_DIR / "lfz" / f"{name}.lfz")]


def py_cmd(name: str) -> list[str]:
    return [sys.executable, str(BENCH_DIR / "python" / f"{name}.py")]


def measure(cmd: list[str], size: int, warmup: int, runs: int) -> dict:
    for _ in range(warmup):
        run_once(cmd, size)
    times: list[float] = []
    out = ""
    err = ""
    rc = 0
    for _ in range(runs):
        dt, out, err, rc = run_once(cmd, size)
        times.append(dt)
    return {
        "median": statistics.median(times),
        "min": min(times),
        "max": max(times),
        "runs": [round(t, 6) for t in times],
        "stdout": out.strip(),
        "stderr": err.strip(),
        "rc": rc,
    }


def tool_version(cmd: list[str]) -> str:
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT))
        return (p.stdout or p.stderr).strip().splitlines()[0]
    except Exception as exc:  # pragma: no cover - best effort
        return f"<unavailable: {exc}>"


def main() -> int:
    ap = argparse.ArgumentParser(description="LFZ vs Python benchmark harness")
    ap.add_argument("--warmup", type=int, default=2)
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--quick", action="store_true", help="first size of each benchmark only")
    args = ap.parse_args()

    if not LFZ_BIN.exists():
        print(f"ERROR: {LFZ_BIN} not found. Run: cargo build --release", file=sys.stderr)
        return 2

    env = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "os": platform.platform(),
        "cpu": platform.processor() or platform.machine(),
        "cpu_count": os.cpu_count(),
        "python": sys.version.split()[0],
        "python_exe": sys.executable,
        "lfz": tool_version([str(LFZ_BIN), "--version"]),
        "rustc": tool_version(["rustc", "--version"]),
        "cargo": tool_version(["cargo", "--version"]),
        "warmup": args.warmup,
        "runs": args.runs,
        "lfz_binary": str(LFZ_BIN.relative_to(ROOT)),
    }

    print("=" * 78)
    print("LFZ vs Python benchmark harness")
    print("=" * 78)
    for k, v in env.items():
        print(f"  {k:14}: {v}")
    print()

    # Startup baseline (noop).
    print("Measuring startup baseline (noop) ...")
    lfz_start = measure([str(LFZ_BIN), "run", str(BENCH_DIR / "fixtures" / "noop.lfz")],
                        0, args.warmup, max(args.runs, 7))
    py_start = measure([sys.executable, str(BENCH_DIR / "fixtures" / "noop.py")],
                       0, args.warmup, max(args.runs, 7))
    env["startup"] = {"lfz_median": lfz_start["median"], "python_median": py_start["median"]}
    print(f"  LFZ    noop median = {lfz_start['median'] * 1000:.2f} ms")
    print(f"  Python noop median = {py_start['median'] * 1000:.2f} ms")
    print()

    results: list[dict] = []
    header = (f"{'benchmark':<16}{'N':>10}{'LFZ med(s)':>12}{'PY med(s)':>12}"
              f"{'ratio':>9}{'LFZ net':>10}{'PY net':>10}{'status':>9}")
    print(header)
    print("-" * len(header))

    for name, (desc, sizes) in BENCHMARKS.items():
        picked = sizes[:1] if args.quick else sizes
        for n in picked:
            lfz = measure(lfz_cmd(name), n, args.warmup, args.runs)
            py = measure(py_cmd(name), n, args.warmup, args.runs)
            same = lfz["stdout"] == py["stdout"] and lfz["rc"] == 0 and py["rc"] == 0
            ratio = (lfz["median"] / py["median"]) if py["median"] > 0 else float("inf")
            lfz_net = max(lfz["median"] - lfz_start["median"], 0.0)
            py_net = max(py["median"] - py_start["median"], 0.0)
            status = "ok" if same else "INVALID"
            print(f"{name:<16}{n:>10}{lfz['median']:>12.5f}{py['median']:>12.5f}"
                  f"{ratio:>9.2f}{lfz_net:>10.5f}{py_net:>10.5f}{status:>9}")
            rec = {
                "benchmark": name,
                "description": desc,
                "size": n,
                "lfz": lfz,
                "python": py,
                "out_equal": same,
                "ratio_median": ratio,
                "lfz_net_median": lfz_net,
                "python_net_median": py_net,
            }
            results.append(rec)
            if not same:
                print(f"    !! output mismatch: LFZ={lfz['stdout']!r} PY={py['stdout']!r}",
                      file=sys.stderr)

    ok = all(r["out_equal"] for r in results)
    print()
    print(f"All outputs matched: {ok}")

    out_dir = BENCH_DIR / "results"
    out_dir.mkdir(exist_ok=True)
    doc = {"environment": env, "results": results}
    (out_dir / "raw.json").write_text(json.dumps(doc, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Raw data written to {out_dir / 'raw.json'}")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
