"""Compare one locally built call-path prototype to the shipped baseline."""
import argparse
import importlib.util
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
BASE_SHA = "76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e"

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("sha256")
    parser.add_argument("work_dir", type=Path)
    parser.add_argument("--baseline", type=Path, default=ROOT / "dist/lfz.exe")
    args = parser.parse_args()
    candidate = args.candidate.resolve()
    baseline = args.baseline.resolve()
    assert baseline.is_relative_to(ROOT)
    assert candidate.is_relative_to(ROOT / "Temp")
    assert len(args.sha256) == 64 and args.sha256 != BASE_SHA
    spec = importlib.util.spec_from_file_location("phase_harness", ROOT / "benchmarks/t25/phases.py")
    driver = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(driver)
    driver.BINARIES = {"baseline": (baseline, BASE_SHA),
                       "prototype": (candidate, args.sha256)}
    sys.argv = [sys.argv[0], "--work-dir", str(args.work_dir)]
    driver.main()

if __name__ == "__main__":
    main()
