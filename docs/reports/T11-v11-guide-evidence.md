# T11-14 人类指南同步与示例证据

日期：2026-10-24。执行者：docs-writer。

## 范围与事实源

仅修改 docs/guide 五文件；引用 docs/spec 的 syntax.md / semantics.md / interface-contract.md（尤其 §10.7 最新复杂度）。69 内置、range 双参、用户追认的文件 IO 已转为已实现。未修改 spec、AI skill、根 README、源码或测试。

团队复验提供 491 项 Rust 测试、130 项黑盒、app 150 次排序零失败、verifier 74 witness 通过；本报告独立证据限定下列指南示例及探针。

## 本次执行产物

- 解释器：target/release/lfz.exe，762880 B，SHA256 `05610b6348efcf9d30fed24b5c434475baec41b6eaa1209d565cca2240a7a64a`。
- 33 个指南 LFZ 代码块逐字提取，26 个成功 exit 0，7 个故意错误教学例 exit 2；全部符合预期。
- 新组合例 tutorial.md 第17块覆盖15个新内置与 range 双参，stdout 与文档输出逐字相同。
- 7 个错误边界探针全部通过，既定错误类与中文消息一致。
- 所有临时脚本及 IO 文件位于项目 Temp/docs-v11/。未发生 abort/panic。

## 可复现方法

从仓库根运行：`python Temp/docs-v11/verify.py`（先设 `PYTHONIOENCODING=utf-8` 使 Windows 控制台可打印 emoji）。脚本仅提取五文件中 ```lfz 代码块，写入 Temp/docs-v11/<文档>-<序号>.lfz，然后调用下述命令，按序号核对退出码。运行组合示例前确保 Temp/docs-v11 已存在。

```powershell
$env:PYTHONIOENCODING="utf-8"
New-Item -ItemType Directory -Force Temp/docs-v11 | Out-Null
python Temp/docs-v11/verify.py
# 每块实际执行形式：
& ./target/release/lfz.exe run Temp/docs-v11/tutorial-17.lfz
```

临时提取器不是交付依赖；下表记录所有原始命令及输出，源代码保留在对应指南代码块，可按序重新保存执行。

## 逐块运行记录

### README.md / LFZ 块 1

命令：`target/release/lfz.exe run Temp\docs-v11\README-1.lfz`；exit 0。

stdout：

```text
Hello, LFZ!
```

stderr：

```text
```

### README.md / LFZ 块 2

命令：`target/release/lfz.exe run Temp\docs-v11\README-2.lfz`；exit 0。

stdout：

```text
你好，LFZ！
这是第 1 次问候。
```

stderr：

```text
```

### tutorial.md / LFZ 块 1

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-1.lfz`；exit 0。

stdout：

```text
Hello, LFZ!
```

stderr：

```text
```

### tutorial.md / LFZ 块 2

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-2.lfz`；exit 0。

stdout：

```text
pi = 3.14, count = 3, type = int
```

stderr：

```text
```

### tutorial.md / LFZ 块 3

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-3.lfz`；exit 0。

stdout：

```text
5 是正数
0..4 的和 = 10
元素 10
元素 20
元素 30
```

stderr：

```text
```

### tutorial.md / LFZ 块 4

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-4.lfz`；exit 0。

stdout：

```text
捕获当轮 cell -> 0
捕获当轮 cell -> 2
捕获当轮 cell -> 4
```

stderr：

```text
```

### tutorial.md / LFZ 块 5

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-5.lfz`；exit 0。

stdout：

```text
fact(5) = 120
fib(10) = 55
```

stderr：

```text
```

### tutorial.md / LFZ 块 6

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-6.lfz`；exit 0。

stdout：

```text
1 2 3
```

stderr：

```text
```

### tutorial.md / LFZ 块 7

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-7.lfz`；exit 0。

stdout：

```text
Alice(93)
true
Alice 现分数 80, isTop=false
1
```

stderr：

```text
```

### tutorial.md / LFZ 块 8

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-8.lfz`；exit 0。

stdout：

```text
总和 = 17
翻倍 = [10, 6, 16, 2]
大于 3 = [5, 8]
升序 = [1, 3, 5, 8]
用 _ 插位: 4
```

stderr：

```text
```

### tutorial.md / LFZ 块 9

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-9.lfz`；exit 0。

stdout：

```text
你好，LFZ！
n=00042  hex=2a  right=    42
pi=3.14
字面量花括号: { } 与美元: ${n}
```

stderr：

```text
```

### tutorial.md / LFZ 块 10

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-10.lfz`；exit 0。

stdout：

```text
[ab   ]
[   ab]
[ ab  ]
[**ab***]
[ab*****]
[00042]
```

stderr：

```text
```

### tutorial.md / LFZ 块 11

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-11.lfz`；exit 0。

stdout：

```text
alpha ： 1
beta ： two
greet ： <fn greet>
Point ： <struct Point>
p ： {x: 1, y: 2}
done
```

stderr：

```text
```

### tutorial.md / LFZ 块 12

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-12.lfz`；exit 0。

stdout：

```text
check 之后继续运行
assert 通过，继续
```

stderr：

```text
check 失败：不相等（软断言，不致命）
```

### tutorial.md / LFZ 块 13

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-13.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
Traceback (most recent call last):
  File "Temp\docs-v11\tutorial-13.lfz", line 2, in <module>
    assert(1 == 2, "一比二大？")
    ^
AssertionError: 断言失败：一比二大？
```

### tutorial.md / LFZ 块 14

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-14.lfz`；exit 0。

stdout：

```text
Student ： <struct Student>
average ： <fn average>
roster ： [{name: "Alice", score: 93}, {name: "Bob", score: 67}, {name: "Cara", score: 88}, {name: "Dan", score: 100}]
名单：
  Alice(93): A
  Bob(67): C
  Cara(88): B
  Dan(100): A

排名：
  1. Dan(100): A
  2. Alice(93): A
  3. Cara(88): B
  4. Bob(67): C

平均分 = 87.00
最高分 = Dan（100）
```

stderr：

```text
```

### tutorial.md / LFZ 块 15

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-15.lfz`；exit 0。

stdout：

```text
len(s) = 3
cs[0] = a
out = ABC
```

stderr：

```text
```

### tutorial.md / LFZ 块 16

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-16.lfz`；exit 0。

stdout：

```text
[ab   ]
```

stderr：

```text
```

### tutorial.md / LFZ 块 17

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-17.lfz`；exit 0。

stdout：

```text
[-2, -1, 0, 1]
2 true
你🙂
你🙂x🙂x
🙂x🙂x你
128578 🙂
0.0 1.0 0.0 1.0
-inf nan inf nan
true
nil
nil
你好🙂
```

stderr：

```text
```

### tutorial.md / LFZ 块 18

命令：`target/release/lfz.exe run Temp\docs-v11\tutorial-18.lfz`；exit 0。

stdout：

```text
名单：
  Alice(93): A 及格
  Bob(67): C 及格
  Cara(88): B 及格
  Dan(100): A 及格
  Eve(55): C 不及格
A 档人数 = 2
平均分 = 80.60
```

stderr：

```text
check 失败：有 1 人不及格
```

### reference.md / LFZ 块 1

命令：`target/release/lfz.exe run Temp\docs-v11\reference-1.lfz`；exit 0。

stdout：

```text
a
a
b
c
```

stderr：

```text
```

### reference.md / LFZ 块 2

命令：`target/release/lfz.exe run Temp\docs-v11\reference-2.lfz`；exit 0。

stdout：

```text
ABC
ABC
```

stderr：

```text
```

### reference.md / LFZ 块 3

命令：`target/release/lfz.exe run Temp\docs-v11\reference-3.lfz`；exit 0。

stdout：

```text
[ab   ]
[   ab]
```

stderr：

```text
```

### reference.md / LFZ 块 4

命令：`target/release/lfz.exe run Temp\docs-v11\reference-4.lfz`；exit 0。

stdout：

```text
false
true

```

stderr：

```text
```

### errors.md / LFZ 块 1

命令：`target/release/lfz.exe run Temp\docs-v11\errors-1.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
  File "Temp\docs-v11\errors-1.lfz", line 2
    let x = 1 $ 2
              ^
SyntaxError: 非法字符 '$'
```

### errors.md / LFZ 块 2

命令：`target/release/lfz.exe run Temp\docs-v11\errors-2.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
Traceback (most recent call last):
  File "Temp\docs-v11\errors-2.lfz", line 5, in <module>
    let r = half(10)
            ^
  File "Temp\docs-v11\errors-2.lfz", line 3, in half
        n / 0
        ^
ZeroDivisionError: 除以零
```

### errors.md / LFZ 块 3

命令：`target/release/lfz.exe run Temp\docs-v11\errors-3.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
File "Temp\docs-v11\errors-3.lfz", line 1
CosmosAnswerError: 你忘记了宇宙的答案
```

### errors.md / LFZ 块 4

命令：`target/release/lfz.exe run Temp\docs-v11\errors-4.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
Traceback (most recent call last):
  File "Temp\docs-v11\errors-4.lfz", line 2, in <module>
    assert(1 == 2, "一比二大？")
    ^
AssertionError: 断言失败：一比二大？
```

### errors.md / LFZ 块 5

命令：`target/release/lfz.exe run Temp\docs-v11\errors-5.lfz`；exit 0。

stdout：

```text
继续运行
```

stderr：

```text
check 失败：软断言示例
```

### testing.md / LFZ 块 1

命令：`target/release/lfz.exe run Temp\docs-v11\testing-1.lfz`；exit 0。

stdout：

```text
```

stderr：

```text
check 失败：软检查不致命
```

### testing.md / LFZ 块 2

命令：`target/release/lfz.exe run Temp\docs-v11\testing-2.lfz`；exit 0。

stdout：

```text
```

stderr：

```text
```

### testing.md / LFZ 块 3

命令：`target/release/lfz.exe run Temp\docs-v11\testing-3.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
Traceback (most recent call last):
  File "Temp\docs-v11\testing-3.lfz", line 3, in <module>
    assert(1 == 2, "这个断言故意失败")
    ^
AssertionError: 断言失败：这个断言故意失败
```

### testing.md / LFZ 块 4

命令：`target/release/lfz.exe run Temp\docs-v11\testing-4.lfz`；exit 2。

stdout：

```text
```

stderr：

```text
File "Temp\docs-v11\testing-4.lfz", line 1
CosmosAnswerError: 你忘记了宇宙的答案
```

## 错误边界探针

每个文件首行为 #42，第二行为下表代码；从仓库根调用 `target/release/lfz.exe run Temp/docs-v11/<name>.lfz`。非法 UTF-8 夹具 invalid.bin 原始字节 FF；no-parent 不存在。

### ord

代码：`ord("a你🙂")`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\ord.lfz", line 2, in <module>
    ord("a你🙂")
    ^
ValueError: ord 的参数必须是单个字符（Unicode 标量数 3）
```

### chr

代码：`chr(55296)`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\chr.lfz", line 2, in <module>
    chr(55296)
    ^
ValueError: chr 的参数不是合法的 Unicode 码点：55296
```

### pad

代码：`padEnd(0, "", "a")`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\pad.lfz", line 2, in <module>
    padEnd(0, "", "a")
    ^
ValueError: 填充串不能为空
```

### utf8

代码：`readFile("Temp/docs-v11/invalid.bin")`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\utf8.lfz", line 2, in <module>
    readFile("Temp/docs-v11/invalid.bin")
    ^
IOError: 无法读取：Temp/docs-v11/invalid.bin（不是合法的 UTF-8 编码）
```

### write-type

代码：`writeFile("Temp/docs-v11/no.txt", 42)`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\write-type.lfz", line 2, in <module>
    writeFile("Temp/docs-v11/no.txt", 42)
    ^
TypeError: 运算符 'writeFile' 不支持 int 与 string
```

### write-missing

代码：`writeFile("Temp/docs-v11/no-parent/no.txt", "a")`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\write-missing.lfz", line 2, in <module>
    writeFile("Temp/docs-v11/no-parent/no.txt", "a")
    ^
IOError: 无法写入：Temp/docs-v11/no-parent/no.txt
```

### append-missing

代码：`appendFile("Temp/docs-v11/no-parent/no.txt", "a")`；exit 2。

```text
Traceback (most recent call last):
  File "Temp\docs-v11\append-missing.lfz", line 2, in <module>
    appendFile("Temp/docs-v11/no-parent/no.txt", "a")
    ^
IOError: 无法追加：Temp/docs-v11/no-parent/no.txt
```

## 文本一致性与复杂度

- 已移除 54 内置、range 仅1参、字符串族未实现、sin/log/exp 不提供等陈旧表述。
- ord 成功 O(1)，错误串完整计数 O(n)；chr O(1)。
- indexOf 最坏 O(n·m)；substring 非空须扫描 UTF-8 前缀 O(min(to,len(s)))；pad* 包含输入计数 O(n+m+输出宽度)。
- contains O(n)，深比较成本另计；避免循环反复扫描。
- 循环字符串加法与循环 push 都为 O(n²)；map 一次构造或预分配下标写入后 join 为 O(片段数+输出大小)。
- UTF-8、cwd、参数须 string、返回值 nil、IOError 与源代码 SyntaxError 的区别已写入。

## 五文件字节数

| 文件 | UTF-8 字节数 |
|---|---:|
| docs/guide/README.md | 7616 |
| docs/guide/tutorial.md | 20577 |
| docs/guide/reference.md | 23630 |
| docs/guide/errors.md | 17693 |
| docs/guide/testing.md | 10264 |
