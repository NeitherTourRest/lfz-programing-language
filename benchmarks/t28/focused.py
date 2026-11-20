"""T28 fixed paired whole-process measurements for immutable append workloads."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import time
import sys

ROOT = Path(__file__).resolve().parents[2]
T27_SHA = "4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636"
ROUNDS = 6
SETTLE_SECONDS = 1


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def make_cases():
    protocol = json.loads((ROOT / "benchmarks/t13/protocol.json").read_bytes())
    cases = {}
    for frozen in protocol["cases"]:
        name = frozen["name"]
        size = frozen["n"]
        if name == "string_ops" or (name == "numeric_loop" and size == 10000):
            source = frozen["source"]
            path = ROOT / f"benchmarks/lfz/{name}.lfz"
            assert path.read_text(encoding="utf8") == source
            assert hashlib.sha256(source.encode()).hexdigest() == frozen["source_sha256"]
            cases[f"{name}_{size}"] = {
                "source": source, "stdin": frozen["stdin"], "stdout": frozen["expected_stdout"],
            }

    for size in (4000, 8000):
        for kind in ("self", "alias"):
            # Reading the old length after assignment makes the old value observable.
            before = "  let old = items\n" if kind == "alias" else ""
            after = "  sum += len(old)\n" if kind == "alias" else ""
            source = (
                "#42\nlet n = int(input())\nvar items = []\nvar i = 0\nvar sum = 0\n"
                f"while i < n {{\n{before}  items = push(i, items)\n{after}  i += 1\n}}\n"
                'print("RESULT n=${len(items)} first=${items[0]} last=${items[n - 1]} sum=${sum}")\n'
            )
            expected_sum = size * (size - 1) // 2 if kind == "alias" else 0
            cases[f"push_{kind}_{size}"] = {
                "source": source, "stdin": f"{size}\n",
                "stdout": f"RESULT n={size} first=0 last={size - 1} sum={expected_sum}\n",
            }
    assert len(cases) == 8
    return cases


def run_sample(command, case, env):
    start = time.perf_counter_ns()
    try:
        proc = subprocess.run(
            command, input=case["stdin"].encode(),
            cwd=ROOT, env=env, capture_output=True, timeout=30,
        )
        out, err, code = proc.stdout, proc.stderr, proc.returncode
        timed_out = False
    except subprocess.TimeoutExpired as exc:
        out, err, code = exc.stdout or b"", exc.stderr or b"", None
        timed_out = True
    elapsed = (time.perf_counter_ns() - start) / 1e6
    return {
        "elapsed_ms": elapsed, "exit": code, "timed_out": timed_out,
        "stdout_hex": out.hex(), "stderr_hex": err.hex(),
        "matched": code == 0 and not err and out.replace(b"\r\n", b"\n") == case["stdout"].encode(),
    }


def measure_python(work, cases, env):
    script = ROOT / "benchmarks/python/string_ops.py"
    python = Path(sys.executable).resolve()
    names = [f"string_ops_{size}" for size in (2000, 4000, 8000)]
    record = {
        "status": "running", "executable": str(python), "version": sys.version,
        "python_sha256": sha(python), "script": script.relative_to(ROOT).as_posix(),
        "script_sha256": sha(script), "runner_sha256": sha(Path(__file__)),
        "lfz_window_sha256": sha(work / "focused.json"),
        "warmup_rounds": 1, "formal_rounds": ROUNDS, "samples": [],
    }
    output = work / "python.json"

    def save():
        pending = work / "python.pending"
        pending.write_bytes((json.dumps(record, ensure_ascii=False, indent=2) + "\n").encode())
        pending.replace(output)

    save()
    try:
        for index in range(-1, ROUNDS):
            offset = (index + 1) % len(names)
            for name in names[offset:] + names[:offset]:
                assert sha(script) == record["script_sha256"]
                sample = run_sample([str(python), "-B", str(script)], cases[name], env)
                sample.update(round=index, case=name)
                record["samples"].append(sample)
                save()
                if not sample["matched"]:
                    raise RuntimeError(f"Python output mismatch in {name}, round {index}")
        assert sha(python) == record["python_sha256"]
        record["median_ms"] = {
            name: statistics.median(s["elapsed_ms"] for s in record["samples"]
                                    if s["round"] >= 0 and s["case"] == name)
            for name in names
        }
        record["status"] = "completed"
        print("Python", json.dumps(record["median_ms"], indent=2))
    except BaseException as exc:
        record.update(status="aborted", error=repr(exc))
        raise
    finally:
        save()



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    args = parser.parse_args()
    work = args.work_dir.resolve()
    if work == ROOT / "Temp" or not work.is_relative_to(ROOT / "Temp"):
        parser.error("--work-dir must be a new directory below project Temp")
    binaries = {"t27": (args.baseline.resolve(), T27_SHA),
                "candidate": (args.candidate.resolve(), args.candidate_sha.lower())}
    if binaries["candidate"][1] == T27_SHA:
        parser.error("candidate must differ from T27")
    for path, expected in binaries.values():
        if sha(path) != expected:
            parser.error(f"binary hash mismatch: {path.name}")
    cases = make_cases()
    work.mkdir(parents=True, exist_ok=False)
    files = {}
    for name, case in cases.items():
        fixture = work / f"{name}.lfz"
        fixture.write_bytes(case["source"].encode())
        files[name] = fixture
        case["sha256"] = sha(fixture)
    record = {
        "status": "running", "runner_sha256": sha(Path(__file__)),
        "warmup_rounds": 1, "formal_rounds": ROUNDS, "settle_seconds": SETTLE_SECONDS,
        "binaries": {label: expected for label, (_, expected) in binaries.items()},
        "cases": cases, "samples": [],
    }
    result = work / "focused.json"

    def save():
        tmp = work / "focused.pending"
        tmp.write_bytes((json.dumps(record, ensure_ascii=False, indent=2) + "\n").encode())
        tmp.replace(result)

    save()
    names = list(cases)
    env = dict(os.environ, TEMP=str(work), TMP=str(work))
    subject = work / "subject.exe"
    try:
        for index in range(-1, ROUNDS):
            labels = list(binaries)
            if index % 2:
                labels.reverse()
            offset = (index + 1) % len(names)
            ordered = names[offset:] + names[:offset]
            for label in labels:
                path, expected = binaries[label]
                shutil.copyfile(path, subject)
                time.sleep(SETTLE_SECONDS)
                for name in ordered:
                    assert sha(subject) == expected and sha(files[name]) == cases[name]["sha256"]
                    sample = run_sample([str(subject), "run", str(files[name])], cases[name], env)
                    sample.update(label=label, case=name, round=index, binary_sha256=expected)
                    record["samples"].append(sample)
                    save()
                    if not sample["matched"]:
                        raise RuntimeError(f"output mismatch in {label}/{name}, round {index}")
        for path, expected in binaries.values():
            assert sha(path) == expected
        medians = {}
        for name in names:
            medians[name] = {}
            for label in binaries:
                times = [s["elapsed_ms"] for s in record["samples"]
                         if s["round"] >= 0 and s["label"] == label and s["case"] == name]
                assert len(times) == ROUNDS
                medians[name][label] = statistics.median(times)
        record.update(status="completed", median_ms=medians)
        print(json.dumps(medians, indent=2))
    except BaseException as exc:
        record.update(status="aborted", error=repr(exc))
        raise
    finally:
        save()


    measure_python(work, cases, env)


if __name__ == "__main__":
    main()
