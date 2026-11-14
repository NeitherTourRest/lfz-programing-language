import hashlib
import json
import os
from pathlib import Path
import subprocess

exe = Path("dist/lfz.exe").resolve()
folder = Path("Temp/T22-lead").resolve()
env = dict(os.environ, TEMP=str(folder), TMP=str(folder))
data = ':unknown\r\ninput()\r\n你好 LFZ\r\n:reset\r\n42\r\n:quit\r\n'.encode()
result = subprocess.run([str(exe), "repl"], input=data, capture_output=True, timeout=12, env=env)
for name, value in (("stdin", data), ("stdout", result.stdout), ("stderr", result.stderr)):
    (folder / (name + ".bin")).write_bytes(value)
record = dict(binary_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(), exit=result.returncode,
              stdout=result.stdout.decode("utf8"), stderr=result.stderr.decode("utf8"))
(folder / "pipe-control.json").write_text(json.dumps(record, ensure_ascii=False, indent=2), encoding="utf8")
assert result.returncode == 2
assert record["stdout"] == "你好 LFZ\n42\n", record
assert record["stderr"] == "repl: 未知命令\n", record
print("UTF-8 pipe control PASS; exit=2")
