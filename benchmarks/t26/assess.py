"""Recompute the T26 diagnostic screen from raw data, without running LFZ."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("raw", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    data = args.raw.read_bytes()
    run = json.loads(data)
    assert run["status"] == "complete"
    assert run["repeats"] == 12 and run["rounds"] == 6
    cases = ("empty", "full", "callbacks", "text_tail", "identity")
    labels = ("baseline", "prototype")
    assert set(run["cases"]) == set(cases)
    assert len(run["samples"]) == 70
    samples = {}
    identities = {}
    for sample in run["samples"]:
        r, label, case = (sample[k] for k in ("round", "label", "case"))
        key = (r,label,case)
        assert -1 <= r < 6 and label in labels and case in cases and key not in samples
        assert sample["exit"] == 0 and not sample["stderr"]
        assert sample["stdout"] == run["cases"][case]["expected_stdout"]
        assert identities.setdefault(label,sample["binary_sha256"]) == sample["binary_sha256"]
        assert sample["elapsed_ms"] >= 0
        samples[key] = sample["elapsed_ms"]
    assert identities["baseline"] == "76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e"
    assert identities["baseline"] != identities["prototype"]
    table = {}
    for case in cases:
        original, changed = ([samples[r,label,case] for r in range(6)] for label in labels)
        before, after = map(statistics.median, (original, changed))
        table[case] = dict(baseline_ms=before, prototype_ms=after, ratio=after/before,
                           faster_pairs=sum(y<x for x,y in zip(original,changed)))
    advance = (table["full"]["ratio"] <= .97 and table["callbacks"]["ratio"] <= .97
               and table["full"]["faster_pairs"] >= 5)
    result = dict(raw_sha256=hashlib.sha256(data).hexdigest(), samples_checked=len(samples),
                  identities=identities, advance_to_r304_window=advance, cases=table)
    with args.output.open("x",encoding="utf8") as out:
        out.write(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result))

if __name__ == "__main__":
    main()
