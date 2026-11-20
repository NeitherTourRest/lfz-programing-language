"""Repeat two frozen R304 workloads in-process and measure array pipeline segments."""
import argparse
import json
import os
from pathlib import Path
import shutil
import statistics
import time

from focused import ROOT, ROUNDS, SETTLE_SECONDS, T27_SHA, run_sample, sha

REPEATS = 16
ARRAY_N = 10000
STRING_N = 2000


def repeated(body, line, size, setup=""):
    # Size and optional segment inputs are computed once. All observed results
    # are printed once per iteration, including those of the control loop.
    source = (
        "#42\nlet n = int(input())\n" + setup +
        f"var rep = 0\nwhile rep < {REPEATS} {{\n" + body +
        "\nrep += 1\n}\n"
    )
    return {"source": source, "stdout": (line + "\n") * REPEATS, "stdin": f"{size}\n"}


def cases_from_protocol(protocol):
    def original(name, size, expected):
        frozen = next(row for row in protocol["cases"] if (row["name"], row["n"]) == (name, size))
        marker = "let n = int(input())\n"
        assert frozen["source"].count(marker) == 1
        assert frozen["expected_stdout"] == expected + "\n" and frozen["stdin"] == f"{size}\n"
        return repeated(frozen["source"].split(marker)[1].rstrip("\n"), expected, size)

    mapped = [(i * 7) % ARRAY_N for i in range(ARRAY_N)]
    selected = [v for v in mapped if v % 2 == 0]
    total = sum(sorted(selected))
    words = [f"item{i}" for i in range(STRING_N)]
    tag = "ab" * 3
    array_line = f"RESULT total={total}"
    string_line = f"RESULT chars={len(','.join(words))} words={len(words)} tag={tag}"
    prefix = "range(n) |> map((x) => x * 7 % n) |> filter((x) => x % 2 == 0)"
    suffix = "sort() |> reduce((a, b) => a + b, 0)"
    return {
        "control": repeated('print("RESULT control=1")', "RESULT control=1", ARRAY_N),
        "array_full": original("array_builtins", ARRAY_N, array_line),
        "array_map_filter": repeated(
            "let ys = " + prefix + '\nprint("RESULT n=${len(ys)} first=${ys[0]} last=${ys[len(ys)-1]}")',
            f"RESULT n={len(selected)} first={selected[0]} last={selected[-1]}", ARRAY_N,
        ),
        "array_sort_reduce": repeated(
            "let total = ys |> " + suffix + '\nprint("RESULT total=${total}")',
            array_line, ARRAY_N, setup="let ys = " + prefix + "\n",
        ),
        "string_full": original("string_map", STRING_N, string_line),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    args = parser.parse_args()
    work = args.work_dir.resolve()
    if work == ROOT / "Temp" or not work.is_relative_to(ROOT / "Temp"):
        parser.error("--work-dir must be a fresh directory below project Temp")
    products = {"t27": (args.baseline.resolve(), T27_SHA),
                "candidate": (args.candidate.resolve(), args.candidate_sha.lower())}
    for path, expected in products.values():
        if sha(path) != expected:
            parser.error(f"binary identity mismatch: {path}")
    if args.candidate_sha.lower() == T27_SHA:
        parser.error("candidate must differ from T27")

    protocol_path = ROOT / "benchmarks/t13/protocol.json"
    cases = cases_from_protocol(json.loads(protocol_path.read_bytes()))
    work.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, TEMP=str(work), TMP=str(work))
    subject = work / "subject.exe"
    for name, case in cases.items():
        fixture = work / (name + ".lfz")
        fixture.write_bytes(case["source"].encode())
        case["sha256"] = sha(fixture)

    record = dict(status="running", repeats=REPEATS, formal_rounds=ROUNDS, warmup_rounds=1,
                  settle_seconds=SETTLE_SECONDS, runner_sha256=sha(Path(__file__)),
                  shared_runner_sha256=sha(Path(__file__).with_name("focused.py")),
                  protocol_sha256=sha(protocol_path), cases=cases,
                  hashes={k: h for k, (_, h) in products.items()}, samples=[])

    def save():
        tmp = work / "diagnosis.pending"
        tmp.write_bytes((json.dumps(record, indent=2) + "\n").encode())
        tmp.replace(work / "diagnosis.json")

    save()
    try:
        for index in range(-1, ROUNDS):
            names = list(cases)
            offset = (index + 1) % len(names)
            labels = list(products)
            if index % 2:
                labels.reverse()
            for label in labels:
                path, expected = products[label]
                shutil.copyfile(path, subject)
                time.sleep(SETTLE_SECONDS)
                for name in names[offset:] + names[:offset]:
                    case = cases[name]
                    fixture = work / (name + ".lfz")
                    assert sha(subject) == expected and sha(fixture) == case["sha256"]
                    sample = run_sample([str(subject), "run", str(fixture)], case, env)
                    sample.update(label=label, round=index, case=name, binary_sha256=expected)
                    record["samples"].append(sample)
                    save()
                    if not sample["matched"]:
                        raise RuntimeError(f"output mismatch in {label}/{name}, round {index}")
        for path, expected in products.values():
            assert sha(path) == expected
        medians = {}
        for name in cases:
            medians[name] = {}
            for label in products:
                values = [s["elapsed_ms"] for s in record["samples"]
                          if s["round"] >= 0 and s["case"] == name and s["label"] == label]
                assert len(values) == ROUNDS
                medians[name][label] = statistics.median(values)
        record.update(status="completed", median_ms=medians)
        print(json.dumps(medians, indent=2))
    except BaseException as exc:
        record.update(status="aborted", error=repr(exc))
        raise
    finally:
        save()


if __name__ == "__main__":
    main()
