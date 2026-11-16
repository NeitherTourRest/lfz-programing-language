"""Recompute the frozen gate without launching LFZ."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("raw", type=Path)
    parser.add_argument("receipt", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    receipt = json.loads(args.receipt.read_bytes())
    raw = json.loads(args.raw.read_bytes())
    assert receipt["status"] == raw["status"] == "completed"
    assert receipt["head"] == "c9196877ea43cc1366de1afa9fa9ec5d1672e4d7"
    assert receipt["sources_before"] == receipt["sources_after"]
    assert receipt["candidate_sha256"] == raw["hashes"]["new"]
    assert receipt["raw_sha256"] == hashlib.sha256(args.raw.read_bytes()).hexdigest()
    reference = ROOT / "benchmarks/t13/assess.py"
    spec = importlib.util.spec_from_file_location("frozen_t13_assessment", reference)
    audit = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(audit)
    audit.RAW = args.raw.resolve()
    sys.argv = [str(reference), "--output", str(args.output)]
    audit.main()

if __name__ == "__main__":
    main()
