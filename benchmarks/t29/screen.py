"""T29 prespecified six-case screen; run only after explicit team-lead dispatch."""
import argparse
import datetime
import json
import os
from pathlib import Path
import shutil
import statistics
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "benchmarks/t28"))
from diagnose import cases_from_protocol
from focused import make_cases, run_sample, sha

T28_SHA = "326b8ec9ede80883c7ac23217068be278ed607e1b84930df4d7f6af6ef9e73ea"
ROUNDS = 6
SETTLE_SECONDS = 1
NAMES = ("control", "array_full", "array_map_filter", "array_sort_reduce", "string_full", "push_self_8000")
CRITERIA = {
    "baseline_sha256": T28_SHA,
    "cases_in_initial_order": list(NAMES),
    "warmup_rounds": 1,
    "formal_rounds": ROUNDS,
    "settle_after_copy_seconds": SETTLE_SECONDS,
    "execution": "one subject.exe path; two binary blocks per round; label order alternates; case order rotates by one",
    "round_indices": "warmup=-1, formal=0..5; odd rounds candidate first, even rounds t28 first",
    "required_improvements": {name: {"candidate_median_strictly_lower": True, "minimum_paired_wins": 4}
                              for name in ("array_full", "string_full")},
    "regression_veto": {name: {"candidate_over_baseline_median_strictly_above": 1.10,
                                "minimum_paired_slower_rounds": 5}
                         for name in ("control", "push_self_8000")},
    "ties": "equal elapsed times count as neither a win nor a slower round",
    "pairing": "same case and formal round; elapsed wall milliseconds, warmup excluded",
    "other_segments": "array_map_filter and array_sort_reduce are diagnostic only, with all medians and paired counts reported",
    "correctness": "every process exit 0, no stderr, stdout exact after CRLF normalization; any mismatch or timeout aborts screen",
    "selection": "pass iff both improvement criteria hold, neither veto holds and every output matches; not the R304 release gate",
    "retries": "no automatic retry, early performance stop, sample removal or threshold change",
}


def build_cases():
    protocol = json.loads((ROOT / "benchmarks/t13/protocol.json").read_bytes())
    cases = cases_from_protocol(protocol)
    assert tuple(cases) == NAMES[:-1]
    cases["push_self_8000"] = make_cases()["push_self_8000"]
    assert tuple(cases) == NAMES
    return cases


def adjudicate(samples):
    rows = {}
    for name in NAMES:
        paired = {}
        for label in ("t28", "candidate"):
            rows_for_label = [s for s in samples if s["case"] == name and s["label"] == label and s["round"] >= 0]
            assert len(rows_for_label) == ROUNDS
            assert {s["round"] for s in rows_for_label} == set(range(ROUNDS))
            paired[label] = {s["round"]: s["elapsed_ms"] for s in rows_for_label}
        old = statistics.median(paired["t28"].values())
        new = statistics.median(paired["candidate"].values())
        wins = sum(paired["candidate"][i] < paired["t28"][i] for i in range(ROUNDS))
        slower = sum(paired["candidate"][i] > paired["t28"][i] for i in range(ROUNDS))
        rows[name] = {
            "t28_median_ms": old, "candidate_median_ms": new,
            "candidate_over_t28_median": new / old,
            "paired_wins": wins, "paired_slower_rounds": slower,
            "paired_ties": ROUNDS - wins - slower,
            "paired_elapsed_ms": [{"round": i, "t28": paired["t28"][i], "candidate": paired["candidate"][i]}
                                  for i in range(ROUNDS)],
        }
    improvements = {name: rows[name]["candidate_median_ms"] < rows[name]["t28_median_ms"]
                    and rows[name]["paired_wins"] >= 4 for name in CRITERIA["required_improvements"]}
    vetoes = {name: rows[name]["candidate_median_ms"] > rows[name]["t28_median_ms"] * 1.10
              and rows[name]["paired_slower_rounds"] >= 5 for name in CRITERIA["regression_veto"]}
    return dict(cases=rows, improvement_pass=improvements, regression_veto=vetoes,
                screen_pass=all(improvements.values()) and not any(vetoes.values()),
                scope="candidate screen only; does not certify R304")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    args = parser.parse_args()
    work = args.work_dir.resolve()
    allowed = ROOT / "Temp/T29"
    if work == allowed or not work.is_relative_to(allowed) or work.exists():
        parser.error("--work-dir must be a fresh subdirectory of project Temp/T29")
    candidate_sha = args.candidate_sha.lower()
    if len(candidate_sha) != 64 or any(c not in "0123456789abcdef" for c in candidate_sha) or candidate_sha == T28_SHA:
        parser.error("candidate SHA must be a distinct 64-character hexadecimal digest")
    products = {"t28": (args.baseline.resolve(), T28_SHA),
                "candidate": (args.candidate.resolve(), candidate_sha)}
    for path, expected in products.values():
        if sha(path) != expected:
            parser.error(f"binary identity mismatch: {path}")
    cases = build_cases()
    dependencies = [Path(__file__), ROOT / "benchmarks/t28/diagnose.py",
                    ROOT / "benchmarks/t28/focused.py", ROOT / "benchmarks/t13/protocol.json"]
    dependency_hashes = {p.relative_to(ROOT).as_posix(): sha(p) for p in dependencies}
    work.mkdir(parents=True, exist_ok=False)
    fixtures = {}
    for name, case in cases.items():
        fixture = work / (name + ".lfz")
        fixture.write_bytes(case["source"].encode())
        fixtures[name] = fixture
        case["sha256"] = sha(fixture)
    record = dict(status="running", started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  criteria=CRITERIA, dependencies_sha256=dependency_hashes, cases=cases,
                  binary_sources={k: {"path": str(p), "sha256": h} for k, (p, h) in products.items()}, samples=[])
    output = work / "screen.json"

    def save():
        pending = work / "screen.pending"
        pending.write_bytes((json.dumps(record, ensure_ascii=False, indent=2) + "\n").encode())
        pending.replace(output)

    env = dict(os.environ, TEMP=str(work), TMP=str(work))
    subject = work / "subject.exe"
    save()
    try:
        for index in range(-1, ROUNDS):
            offset = (index + 1) % len(NAMES)
            ordered = list(NAMES[offset:] + NAMES[:offset])
            labels = list(products)
            if index % 2:
                labels.reverse()
            for label in labels:
                path, expected = products[label]
                assert sha(path) == expected
                shutil.copyfile(path, subject)
                time.sleep(SETTLE_SECONDS)
                for name in ordered:
                    case = cases[name]
                    assert sha(subject) == expected and sha(fixtures[name]) == case["sha256"]
                    for dependency in dependencies:
                        assert sha(dependency) == dependency_hashes[dependency.relative_to(ROOT).as_posix()]
                    command = [str(subject), "run", str(fixtures[name])]
                    sample = run_sample(command, case, env)
                    sample.update(label=label, round=index, case=name, binary_sha256=expected,
                                  fixture_sha256=case["sha256"], command=command,
                                  stdin_hex=case["stdin"].encode().hex(),
                                  observed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
                    record["samples"].append(sample)
                    save()
                    if not sample["matched"]:
                        raise RuntimeError(f"output mismatch or timeout in {label}/{name}, round {index}")
        for path, expected in products.values():
            assert sha(path) == expected
        assert len(record["samples"]) == 2 * len(NAMES) * (ROUNDS + 1)
        record.update(status="completed", assessment=adjudicate(record["samples"]))
        print(json.dumps(record["assessment"], indent=2))
    except BaseException as exc:
        record.update(status="aborted", error=repr(exc))
        raise
    finally:
        record["finished_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        save()


if __name__ == "__main__":
    main()