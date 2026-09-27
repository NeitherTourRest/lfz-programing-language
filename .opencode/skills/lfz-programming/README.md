# lfz-programming — LFZ 语言的 AI 编程 Skill 包

> 一个 **Skill**：让**从未接触过 LFZ** 的编程 Agent，只凭本包就能写出**正确、可运行**的 `.lfz` 程序。
> 事实源：`docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1）。本包只做**速查 / 转述**，与 spec 冲突时**以 spec 为准**。

---

## 1. 它是什么 / 给谁用 / 何时加载

| 项 | 说明 |
|---|---|
| **是什么** | 面向 **AI 编程 Agent** 的 LFZ 语言速查指南 + 提示模板 + 实测记录 + 安装说明（纯 Markdown，无运行时依赖） |
| **给谁用** | 需要**写 / 改 / 调试 `.lfz` 程序**的任意 Agent（app-dev 是首位真实用户） |
| **何时加载** | 任务涉及 `.lfz` 文件、LFZ 脚本、`lfz run`、`cargo run -- run`，或回答 LFZ 语法/语义问题时 |
| **如何加载** | opencode 读取 `SKILL.md` 的 frontmatter（`name` / `description`），**自动匹配并在需要时载入** |

**包内文件**

| 文件 | 作用 |
|---|---|
| `SKILL.md` | **核心**：`#42` 铁律 + **Agent 工作流（5 步）** + 12 错误类 + 语法速查 + 24 陷阱 + **错误→修法对照表** + **§4.6 O(n) 构建性能红线** + 54 内置 + 分级示例 L1–L5 + 自检清单 + 提示模板 |
| `README.md` | 本文件 —— **安装与使用（最后一公里）** |
| `prompt-template.md` | 复制即用的 prompt 骨架（新写 / 审查两类）+ 最小任务基线 |
| `VERIFICATION.md` | **实测证据**：交付要求对照表（任务要求 + spec §11.1 五条）+ 4 个实跑程序（含输出与退出码） |

---

## 2. 安装方式（三选一）

### 方式 ① 本项目（已就位，无需安装）

本仓库已把 skill 放在 `.opencode/skills/lfz-programming/`；opencode **自动扫描**项目内的 `.opencode/skills/`。

- **生效条件**：**重启 opencode**（或重开会话），让 skill 加载器重新扫描该目录。
- **验证**：重启后让 Agent 执行"用 LFZ 写一个 hello world"，Agent 应能自动命中本 skill。

### 方式 ② 全局安装（所有项目可用）

把整个目录复制到用户级 skills 目录：

```powershell
# 目标目录（Windows）：%USERPROFILE%\.config\opencode\skills\lfz-programming
$dest = "$env:USERPROFILE\.config\opencode\skills\lfz-programming"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item -Recurse -Force ".\.opencode\skills\lfz-programming\*" $dest
```

- 其他平台路径：`~/.config/opencode/skills/lfz-programming/`。
- 复制后**重启 opencode**；此后**任意项目**里涉及 `.lfz` 的任务都会命中本 skill。

### 方式 ③ 其他 harness（如 DSH）

本包是**纯文件**（Markdown），任何支持"文件系统 skill"的 harness 都能挂载。以 DSH 的 `dsh-skill-filesystem` 为例，指向本目录即可：

```jsonc
// 示意：把 skill 目录注册给 dsh-skill-filesystem（字段名以你的 harness 文档为准）
{
  "mcpServers": {
    "dsh-skill-filesystem": {
      "command": "<你的 dsh 可执行文件>",
      "args": [
        "--skill-dir",
        "D:\\XUE\\2026fall\\Program Design\\lfz-programing language design\\.opencode\\skills\\lfz-programming"
      ]
    }
  }
}
```

> 本 skill 只依赖**LFZ 解释器**来做"实跑验证"（见 §3），不依赖任何 opencode 专属插件，故可迁移。

---

## 3. 运行前提（重要）

`SKILL.md` 的**第 4 步（实跑）**要求能运行 LFZ。三种等价入口，任选其一：

| 入口 | 命令 | 前提 |
|---|---|---|
| 仓库源码 | `cargo run --quiet -- run <file.lfz>` | 已装 Rust 工具链；**`cargo` 在 PATH** |
| 发布二进制 | `lfz run <file.lfz>`（或 `.\dist\lfz.exe run <file.lfz>`） | `dist\lfz.exe` 在 PATH，或用其完整路径 |
| 裸文件调用 | `lfz <file.lfz>`（等价 `lfz run <file.lfz>`） | 同上 |

**`cargo` 的 PATH 坑（Windows）**：新开的 shell 常找不到 `cargo`，先执行：

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

**入口限制（v1 实测）**：CLI 只有 `lfz run <file>`、`lfz <file>`、`lfz test`；**没有** `-e` / stdin 管道 / REPL。程序一律**先落成 `.lfz` 文件再运行**。
**退出码（按语境区分）**：`0` 成功；**`lfz run`** 下 `assert`/`fail` 失败 → `2`；**`lfz test`** 下有用例失败 → `1`；CLI 参数错误 / 语法或运行时错误 → `2`。

---

## 4. 它不做什么（边界）

- **不定义、不发明语法**：一切语法/语义以 `docs/spec/` 为准；本包与 spec 冲突时**以 spec 为准**。
- **不替你改解释器**：若怀疑解释器行为与 spec 不符，报告维护者，**不要**按"另一门语言的直觉"改代码。
- **不是人类教程**：面向人的手册在 `docs/guide/`（docs-writer 维护）；本包**只面向 AI**。
- **不含编号错误码**：LFZ 只有**类名 + 中文消息**，禁止使用 / 期待形如 `E-xxx` 的旧错误码。

---

## 5. 30 秒自检（验证安装是否成功）

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
"#42`nprint(""Hello, LFZ!"")" | Set-Content -Encoding utf8 .\hello.lfz
cargo run --quiet -- run .\hello.lfz   # 期望输出 Hello, LFZ!
$LASTEXITCODE                          # 期望 0
```

（实测输出 `Hello, LFZ!`，退出码 `0`；`.lfz` 首行 `#42` 即使带 UTF-8 BOM 也合法。）

---

## 6. 证据与出处

- **实测记录**（4 程序 + 19 类错误场景 + 头部边界）：[`VERIFICATION.md`](./VERIFICATION.md)。
- **T11-③ D1–D7 修订实测**（字符串不可下标 / `range` 签名 / 循环 `let` / 退出码 / 对齐 / `len(string)` / 隐性语法 + O(n) 构建）：[`VERIFICATION.md`](./VERIFICATION.md) §8。
- **交付要求对照**（任务要求第 4 条 + spec §11.1 五条硬性要求 → SKILL 落点）：[`VERIFICATION.md`](./VERIFICATION.md) §1。
- **语法 / 语义 / 接口契约**（唯一事实源）：`docs/spec/`。
