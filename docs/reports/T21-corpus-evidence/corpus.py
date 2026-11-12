import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path.cwd()
exe = (root / "target/release/lfz.exe").resolve()
folder = root / "Temp/T21-fmt"
sha = lambda data: hashlib.sha256(data).hexdigest()
expected = sys.argv[1].lower()
assert sha(exe.read_bytes()) == expected
files = subprocess.check_output(["git", "ls-files", "-z", "--", "examples", "app", "tests"]).decode().split("\0")
rows = []
for path in sorted(p for p in files if p.endswith(".lfz")):
    source = root / path
    original = source.read_bytes()
    result = subprocess.run([str(exe), "fmt", str(source)], capture_output=True, timeout=12)
    assert source.read_bytes() == original, path
    row = {"file": path, "input_sha256": sha(original), "exit": result.returncode}
    if result.returncode == 0:
        assert not result.stderr, path
        formatted = folder / "corpus" / path
        formatted.parent.mkdir(parents=True, exist_ok=True)
        formatted.write_bytes(result.stdout)
        second = subprocess.run([str(exe), "fmt", str(formatted)], capture_output=True, timeout=12)
        assert second.returncode == 0 and not second.stderr and second.stdout == result.stdout, path
        row.update(changed=result.stdout != original, output_sha256=sha(result.stdout), idempotent=True)
    else:
        assert result.returncode == 2 and not result.stdout and result.stderr, path
        row["diagnostic"] = result.stderr.decode("utf8", errors="replace")
        assert "保真校验" not in row["diagnostic"], path
    rows.append(row)
assert sha(exe.read_bytes()) == expected
summary = dict(candidate=expected, cases=len(rows), accepted=sum(r["exit"] == 0 for r in rows),
               changed=sum(r.get("changed", False) for r in rows),
               rejected=sum(r["exit"] == 2 for r in rows), rows=rows)
with (folder / "corpus.json").open("x", encoding="utf8") as out:
    json.dump(summary, out, ensure_ascii=False, indent=2)
print(json.dumps({key: value for key, value in summary.items() if key != "rows"}))
