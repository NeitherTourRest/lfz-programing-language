# T11-③（人类向文档）实跑证据 — D1–D7 + 架构师裁定修正

> 作者: docs-writer ｜ 日期: 2026-09-27 ｜ 范围: `docs/guide/**`（人类向手册；**未改** `.opencode/skills/**`、`docs/spec/**`、`src/**`、`tests/**`、`app/**`）
> 依据: `.opencode/team/FEATURE-AUDIT.md` §6.2（D1–D7）/§7.1；`docs/reports/conformance-B-semantics.md`（obs-B-01/02）；事实源 `docs/spec/**`（只读）
> 解释器: `dist\lfz.exe`（`lfz 1.0.1`，710144 B，由当前源码同源重建）

---

## 1. 命令模板

所有探针位于 `Temp/docs-d1d7/`，用 cmd 重定向取干净 UTF-8 输出：

```
dist\lfz.exe run <file> > out\<name>.out 2>&1
```

---

## 2. 逐条证据（实测输出 + 退出码）

### D1 字符串不可下标 + `split("", s)` 惯用法（O(n)）

`d1_sindex.lfz`（`let s = "abc"` + `print(s[0])`）→ **exit 2**：
```text
Traceback (most recent call last):
  File "Temp\docs-d1d7\d1_sindex.lfz", line 3, in <module>
    print(s[0])
          ^
TypeError: 运算符 '[]' 不支持 string 与 array / struct
```

`d1_split.lfz`（`split("", "abc")`）→ **exit 0**：
```text
len(s) = 3
split("", s) = ["a", "b", "c"]
cs[0] = a
重组 = abc
```

### D2 `range` 精确签名（仅 1 参）

`d2_range.lfz` → **exit 0**：
```text
range(3) = [0, 1, 2]
range(0) = []
range(-2) = []
```
`d2_range2.lfz`（`range(1, 4)`）→ **exit 2**：
```text
TypeError: 函数 range 期待 1 个参数，得到 2
```

### D3 循环体 `let` 每轮新绑定

`d3_letloop.lfz` → **exit 0**：
```text
捕获当轮 cell -> 0
捕获当轮 cell -> 2
捕获当轮 cell -> 4
本轮 y = 10
本轮 y = 11
本轮 y = 12
```

### D5 对齐 `<`/`>`/`^` + fill 可用；动态宽度不支持

`d5_align.lfz` → **exit 0**：
```text
[ab   ]
[   ab]
[ ab  ]
[**ab***]
[ab*****]
[00042]
```
`d5_width_dyn.lfz`（`"${"ab":>w}"`）→ **exit 2**：
```text
ValueError: 格式说明符非法：'>w'
```

### D6 `len(string)` 合法（Unicode 标量数）

`d6_lenstr.lfz` → **exit 0**：
```text
len("abc") = 3
len("你好") = 2
len("a你b") = 3
type(len("abc")) = int
```

### D7 隐性语法正面示例

`d7_misc.lfz` → **exit 0**：
```text
链式下标赋值 t[1][0]=7 -> [[0, 0], [7, 0]]
else if 命中 -> B
多插值: 1+2=3，再来一个 2
零参 print 输出一个空行：

（上一行是空行）
&& 短路: false
|| 短路: true
```

### O(n) 字符串构建：`push` + `join`

`on_join.lfz` → **exit 0**：`push + join -> ABC`

### float 显示（最短往返优先、`.0` 仅定点）

`floatfmt.lfz` → **exit 0**：
```text
1.0 -> 1.0
100.0 -> 100.0
-0.0 -> -0.0
1.5 -> 1.5
1e16 -> 1e16
1e20 -> 1e20
1e-5 -> 1e-5
1.2345678901234568e17 -> 1.2345678901234568e17
```

### 新增细分消息

- `err_capacity.lfz`（`repeat(4611686018427387904, "ab")`）→ **exit 2**：
  ```text
  Traceback (most recent call last):
    File "Temp\docs-d1d7\err_capacity.lfz", line 2, in <module>
      let r = repeat(4611686018427387904, "ab")
              ^
  OverflowError: 容量溢出：所需容量超出可分配上限
  ```
- `err_nesting.lfz`（`(`×1001）→ **exit 2**，末行：`SyntaxError: 嵌套深度超限（超过 1000 层）`
- `err_exprdepth.lfz`（`1+`×10000）→ **exit 2**，末行：`SyntaxError: 表达式嵌套过深（超过 10000 层）`

### 零帧运行期错误（obs-B-01 裁定）

`lfz run Temp\docs-d1d7\nope_missing.lfz`（文件不存在）→ **exit 2**，**仅一行、无 Traceback 头**：
```text
IOError: 无法读取：Temp\docs-d1d7\nope_missing.lfz
```

---

## 3. 文档内示例二次验证（原文逐字复跑）

| 文档位置 | 探针 | 结果 |
|---|---|---|
| tutorial Step 13（`s13.lfz`） | `doc_s13.lfz` | exit 0；`len(s) = 3` / `cs[0] = a` / `out = ABC` |
| tutorial Step 3 补充（`s3b.lfz`） | `doc_s3b.lfz` | exit 0；捕获当轮 cell -> 0 / 2 / 4 |
| tutorial Step 8（`s8b.lfz`） | `doc_s8b.lfz` | exit 0；六行对齐输出全中 |
| reference §3.5 / tutorial Step 13（`padRight`） | `doc_padRight.lfz` | exit 0；`[ab   ]` |
| reference §3.5 ① | `doc_ref35_1.lfz` | exit 0；`a` / `a` / `b` / `c` |
| reference §3.6 | `doc_ref36.lfz` | exit 0；`false` / `true` / 空行 |

**结论：文档中所有新增 LFZ 示例均以 `dist\lfz.exe` 实跑复核，输出与文档所载逐字一致。**

---

## 4. 变更量（`git diff --stat` 与字节数）

```
docs/guide/README.md    |  10 ++++-
docs/guide/errors.md    |  51 ++++++++++++++++++++---
docs/guide/reference.md | 108 ++++++++++++++++++++++++++++++++++++++++++++++--
docs/guide/testing.md   |  13 ++++--
docs/guide/tutorial.md  |  99 ++++++++++++++++++++++++++++++++++++++++++--
5 files changed, 265 insertions(+), 16 deletions(-)
```

| 文件 | 修改后字节数 | 行数 |
|---|---|---|
| `docs/guide/README.md` | 7153 | 150 |
| `docs/guide/tutorial.md` | 18271 | 622 |
| `docs/guide/reference.md` | 20825 | 338 |
| `docs/guide/errors.md` | 16595 | 317 |
| `docs/guide/testing.md` | 9696 | 231 |

其它一致性：`lfz test` 实测 **90/90 exit 0**（guide 内 `82` 陈旧计数已同步为 `90`）；`#42` 审计：guide 共 **32** 个 `lfz` 代码块，**30** 个以 `#42` 开头，2 个例外为**故意缺前导的负例**（`errors.md` e3、`testing.md` fixtures/no_header，均已在文内标注）。
