"""Measure join independently and compare first-match search with full filtering."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
SIZE = 16000
REPEATS = 40
ROUNDS = 8


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def make_cases(directory):
    setup = f'let parts = range({SIZE}) |> map((i) => "item${{i}}")\n'
    joined_size = sum(len(f"item{i}") for i in range(SIZE)) + SIZE - 1
    sources = {
        "join": (setup, 'len(join(",", parts))', joined_size),
        "join_control": (setup, "len(parts)", SIZE),
        "find_first": (f"let items = range({SIZE})\n", "items |> findIndex((x) => x == 0)", 0),
        "filter_first": (f"let items = range({SIZE})\n", "(items |> filter((x) => x == 0))[0]", 0),
    }
    cases = {}
    for name, (setup, expression, value) in sources.items():
        source = (
            f"#42\n{setup}var total = 0\nvar completed = 0\n"
            f"while completed < {REPEATS} {{\n  total += {expression}\n"
            '  completed += 1\n}\nprint(total, completed)\n'
        )
        fixture = directory / f"{name}.lfz"
        fixture.write_text(source, encoding="utf8", newline="\n")
        cases[name] = {"source": source, "sha256": digest(fixture),
                       "stdout": f"{REPEATS * value} {REPEATS}\n"}
    return cases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--feature", type=Path, required=True)
    parser.add_argument("--feature-sha", required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    args = parser.parse_args()
    work = args.work_dir.resolve()
    binaries = {"feature": (args.feature.resolve(), args.feature_sha),
                "candidate": (args.candidate.resolve(), args.candidate_sha)}
    if work == ROOT / "Temp" or not work.is_relative_to(ROOT / "Temp"):
        parser.error("--work-dir must be below project Temp")
    work.mkdir(parents=True, exist_ok=False)
    for path, sha in binaries.values():
        assert digest(path) == sha

    cases = make_cases(work)
    subject = work / "subject.exe"
    output = work / "focused.json"
    record = {"status": "running", "array_size": SIZE, "repeats": REPEATS,
              "rounds": ROUNDS, "warmups": 1, "settle_seconds": 1,
              "hashes": {label: sha for label, (_, sha) in binaries.items()},
              "cases": cases, "samples": []}

    def save():
        output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf8")

    save()
    try:
        for round_index in range(-1, ROUNDS):
            labels = list(binaries)
            if round_index % 2:
                labels.reverse()
            for label in labels:
                path, sha = binaries[label]
                if round_index >= 0:
                    assert digest(path) == sha
                shutil.copyfile(path, subject)
                time.sleep(1)
                names = list(cases) if label == "candidate" else ["join", "join_control"]
                offset = round_index % len(names)
                for name in names[offset:] + names[:offset]:
                    fixture = work / f"{name}.lfz"
                    expected = cases[name]
                    assert digest(subject) == sha and digest(fixture) == expected["sha256"]
                    start = time.perf_counter_ns()
                    completed = subprocess.run(
                        [str(subject), "run", str(fixture)],
                        cwd=ROOT, capture_output=True, timeout=30,
                    )
                    elapsed_ms = (time.perf_counter_ns() - start) / 1e6
                    sample = {"round": round_index, "binary": label, "case": name,
                              "elapsed_ms": elapsed_ms, "exit": completed.returncode,
                              "stdout": completed.stdout.decode("utf8").replace("\r\n", "\n"),
                              "stderr": completed.stderr.decode("utf8", "replace")}
                    record["samples"].append(sample)
                    save()
                    assert sample["exit"] == 0 and sample["stdout"] == expected["stdout"], sample
                    assert not sample["stderr"], sample
        record["median_ms"] = {
            label: {name: statistics.median(values) for name in cases
                    if (values := [s["elapsed_ms"] for s in record["samples"]
                                   if s["round"] >= 0 and s["binary"] == label and s["case"] == name])}
            for label in binaries
        }
        record["status"] = "completed"
        print(json.dumps(record["median_ms"], indent=2))
    except BaseException:
        record["status"] = "aborted"
        raise
    finally:
        save()


if __name__ == "__main__":
    main()
