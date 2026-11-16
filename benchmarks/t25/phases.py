"""Focused phase measurements. This does not evaluate the R304 gate."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
WORK = ROOT / "Temp/T25-perf"

BINARIES = {
    "t12": (ROOT / "Temp/t13-perf/lfz-t12.exe", "47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f"),
    "current": (ROOT / "dist/lfz.exe", "76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e"),
}
N = 8000
REPEATS = 12
ROUNDS = 6

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def make_cases(fixtures):
    pristine = [f"item{i}" for i in range(N)]
    joined = ",".join(pristine)
    text_input = fixtures / "cleaned.txt"
    text_input.write_bytes("\n".join(pristine).encode())
    callback = '''let parts = range(8000) |> map((i) => "  item${i}  ")
 let cleaned = parts |> map((s) => trim(s))
'''
    tail = '''let joined = join(",", cleaned)
 let words = split(",", replace("item", "ITEM", joined))
'''
    body = {
        "empty": ("", "total += 1", REPEATS),
        "full": (callback + " " + tail, "total += len(joined) + len(words)", REPEATS * (len(joined) + N)),
        "callbacks": (callback, "total += len(cleaned) + len(cleaned[7999])", REPEATS * (N + len(pristine[-1]))),
        "text_tail": (tail, "total += len(joined) + len(words)", REPEATS * (len(joined) + N)),
        "identity": ("let copied = numbers |> map((i) => i)\n", "total += len(copied) + copied[7999]", REPEATS * (N + N - 1)),
    }
    cases = {}
    for name, (work, checksum, expected) in body.items():
        setup = ""
        if name == "text_tail":
            setup = 'let cleaned = split("\\n", readFile(' + json.dumps(text_input.relative_to(ROOT).as_posix()) + '))\n'
        if name == "identity":
            setup = "let numbers = range(8000)\n"
        source = "#42\n" + setup + f"var total = 0\nvar r = 0\nwhile r < {REPEATS} {{\n " + work
        source += " " + checksum + '\n r += 1\n}\nprint("RESULT total=${total}")\n'
        path = fixtures / (name + ".lfz")
        path.write_bytes(source.encode())
        cases[name] = (path, f"RESULT total={expected}\n", source)
    return cases

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=Path, default=WORK)
    work = parser.parse_args().work_dir.resolve()
    if not work.is_relative_to(ROOT / "Temp") or work == ROOT / "Temp":
        parser.error("--work-dir must be a directory below project Temp")
    fixtures = work / "fixtures"
    output = work / "phase-window.json"
    assert not output.exists()
    fixtures.mkdir(parents=True, exist_ok=True)
    for path, expected in BINARIES.values():
        assert sha(path) == expected
    cases = make_cases(fixtures)
    names = list(cases)
    subject = work / "subject.exe"
    record = dict(status="running", repeats=REPEATS, rounds=ROUNDS, cases={
        n: dict(source=s, sha256=sha(p), expected_stdout=e) for n, (p,e,s) in cases.items()},
        cleaned_input_sha256=sha(fixtures / "cleaned.txt"), samples=[])
    def save():
        output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf8")
    save()
    try:
        for r in range(-1, ROUNDS):
            labels = list(BINARIES)
            if r % 2:
                labels.reverse()
            for label in labels:
                path, expected_hash = BINARIES[label]
                shutil.copyfile(path, subject)
                time.sleep(2)
                for offset in range(len(names)):
                    name = names[(offset+r+1) % len(names)]
                    fixture, expected_stdout, _ = cases[name]
                    assert sha(subject) == expected_hash and sha(fixture) == record["cases"][name]["sha256"]
                    start = time.perf_counter_ns()
                    result = subprocess.run([str(subject), "run", str(fixture)], cwd=ROOT, capture_output=True, timeout=15)
                    elapsed = (time.perf_counter_ns() - start) / 1e6
                    stdout = result.stdout.decode("utf8").replace("\r\n", "\n")
                    record["samples"].append(dict(round=r, label=label, case=name, binary_sha256=expected_hash,
                        elapsed_ms=elapsed, exit=result.returncode, stdout=stdout, stderr=result.stderr.decode("utf8", "replace")))
                    save()
                    assert result.returncode == 0 and stdout == expected_stdout and not result.stderr, record["samples"][-1]
        record["status"] = "complete"
    except BaseException:
        record["status"] = "aborted"
        raise
    finally:
        save()
    for name in names:
        medians = [statistics.median(x["elapsed_ms"] for x in record["samples"] if
                    x["round"] >= 0 and x["case"] == name and x["label"] == label) for label in BINARIES]
        print(name, *(round(m, 3) for m in medians), round(medians[1]/medians[0], 3))

if __name__ == "__main__":
    main()
