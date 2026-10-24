# T11-14：v1.1 已实现特性 skill 同步证据

- 日期：2026-10-24；执行者 ai-dx-engineer；事实源 `docs/spec/interface-contract.md` §10.7（最新 ord/substring/pad 复杂度订正）。
- 当前二进制：`target/release/lfz.exe`，762880 B，版本号仍 1.0.1；能力按实跑确认，不以版本号推断。
- 同步 skill 四文件；69 内置 = 核心/数组22 + struct5 + 字符串13 + 数值/转换/IO/断言29。新名字15个，range重载不另计名字。
- 保留 D1–D7；D2 旧「只接受单参」替换，VERIFICATION §8 标为历史。未实施新 Agent 八题盲测，不能声称盲测通过率提升。
- 临时件均项目 `Temp/ai-dx-v11/`；下文保存增量源码与错误观测。

## 1. 完整增量示例

从仓库根目录执行（PowerShell），预先创建 `Temp/ai-dx-v11/`，保存下方代码至该目录 `probe.lfz`（UTF-8）：

```powershell
New-Item -ItemType Directory -Force Temp/ai-dx-v11 | Out-Null
& ./target/release/lfz.exe run Temp/ai-dx-v11/probe.lfz
$LASTEXITCODE
```

真实 stdout：`v11 skill OK`；退出码 **0**，20 条 assert 全通过。覆盖写的 `Temp/ai-dx-v11/text.txt` 为 UTF-8 `日本語😀`；脚本在子目录，路径仍基于仓库 CWD。未访问项目外路径。
```lfz
#42
assert(range(-2, 2) == [-2, -1, 0, 1])
assert(range(3, 3) == [])
assert(indexOf("語", "日本語😀") == 2)
assert(indexOf("?", "日本語") == -1)
assert(indexOf("", "日本語") == 0)
assert(endsWith("😀", "日本語😀"))
assert(substring(-2, 3, "日本語😀") == "日本語")
assert(substring(1, 99, "日本語😀") == "本語😀")
assert(padEnd(5, "😀語", "日") == "日😀語😀語")
assert(padStart(4, "😀語", "日") == "😀語😀日")
assert(ord("😀") == 128512)
assert(chr(128512) == "😀")
assert(contains({ k: [1] }, [{ k: [1] }]))
assert(sin(0) == 0.0 && cos(0) == 1.0)
assert(log(1) == 0.0 && exp(0) == 1.0)
assert(log(0) == float("-inf"))
assert(exp(1000) == float("inf"))
let p = "Temp/ai-dx-v11/text.txt"
assert(writeFile(p, "日本語") == nil)
assert(appendFile(p, "😀") == nil)
assert(readFile(p) == "日本語😀")
print("v11 skill OK")
```

## 2. 错误边界与 L1–L5 复跑

错误探针均首行 `#42` + 下文单条表达式，执行 `& ./target/release/lfz.exe run --json Temp/ai-dx-v11/<name>.lfz`。非法 UTF-8 夹具字节为 `FF FE`；no-parent 父目录不存在。L1–L5 从当前 SKILL §7 按 lfz 代码块逐块提取至 Temp 后运行。

| 探针 | 退出码 | 真实输出/消息 |
| -- | -- | -- |
| ord-empty | 2 | ValueError: ord 的参数必须是单个字符（Unicode 标量数 0） |
| ord-many | 2 | ValueError: ord 的参数必须是单个字符（Unicode 标量数 3） |
| chr-surrogate | 2 | ValueError: chr 的参数不是合法的 Unicode 码点：55296 |
| chr-too-high | 2 | ValueError: chr 的参数不是合法的 Unicode 码点：1114112 |
| pad-empty | 2 | ValueError: 填充串不能为空 |
| pad-start-empty | 2 | ValueError: 填充串不能为空 |
| io-missing | 2 | IOError: 无法读取：Temp/ai-dx-v11/does-not-exist.txt |
| io-invalid-utf8 | 2 | IOError: 无法读取：Temp/ai-dx-v11/invalid.bin（不是合法的 UTF-8 编码） |
| io-write-missing-parent | 2 | IOError: 无法写入：Temp/ai-dx-v11/no-parent/out.txt |
| io-append-missing-parent | 2 | IOError: 无法追加：Temp/ai-dx-v11/no-parent/out.txt |
| io-type | 2 | TypeError: 运算符 'writeFile' 不支持 int 与 string |
| range-three | 2 | TypeError: 函数 range 期待 2 个参数，得到 3 |
| string-index | 2 | TypeError: 运算符 '[]' 不支持 string 与 array / struct |
| L1 | 0 | Hello, LFZ! |
| L2 | 0 | hello, LFZ<br>7 / 2 = 3.5<br>div(7, 2) = 3<br>-7 % 3 = 2<br>round(2.5) = 2<br>round(3.5) = 4<br>grade = high<br>name ： LFZ<br>count ： 4<br>grade ： high |
| L3 | 0 | ranking (desc):<br>  Alice:  93<br>  Cara:  88<br>  Bob:  67<br>members: Alice, Bob, Cara<br>average: 82.67<br>top: Alice (93) |
| L4 | 0 | total=13, distinct=8<br>top 3 words (desc):<br>  the:  4<br>  fox:  2<br>  jumps:  2 |
| L5 | 0 | ["L", "F", "Z"]<br>L<br>Z<br>L<br>F<br>Z<br>[ab  \|    7\|**ab**]<br>[c   \|  128\|**c***]<br>small<br><br>false<br>true |
| baseline-ieee | 0 | 6<br>nan nan nan |

负例表达式（每个单独文件）：

```lfz
// ord-empty
ord("")
// ord-many
ord("日本語")
// chr-surrogate
chr(55296)
// chr-too-high
chr(1114112)
// pad-empty
padEnd(0, "", "abc")
// pad-start-empty
padStart(0, "", "abc")
// io-missing
readFile("Temp/ai-dx-v11/does-not-exist.txt")
// io-invalid-utf8
readFile("Temp/ai-dx-v11/invalid.bin")
// io-write-missing-parent
writeFile("Temp/ai-dx-v11/no-parent/out.txt", "x")
// io-append-missing-parent
appendFile("Temp/ai-dx-v11/no-parent/out.txt", "x")
// io-type
writeFile("Temp/ai-dx-v11/text.txt", 42)
// range-three
range(1, 4, 1)
// string-index
"abc"[0]
```

最小任务基线与 IEEE 探针完整源码：

```lfz
#42
fn sum_even(xs) {
    xs |> filter((x) => x % 2 == 0) |> reduce((acc, x) => acc + x, 0)
}
assert(sum_even([1,2,3,4,5]) == 6)
assert(len("日本語") == 3 && len("😀") == 1)
print(sum_even([1,2,3,4,5]))
print(sin(float("inf")), cos(float("nan")), log(-1))
```

## 3. 身份与结论边界

二进制 SHA256：05610B6348EFCF9D30FED24B5C434475BAEC41B6EAA1209D565CCA2240A7A64A。本轮亲测增量示例20条断言、13个预期错误、L1–L5、最小任务基线和 IEEE 边界均符合观测。全量编译/491 Rust 单测/130黑盒由 team-lead/verifier 汇总，本报告不代替独立验收。
