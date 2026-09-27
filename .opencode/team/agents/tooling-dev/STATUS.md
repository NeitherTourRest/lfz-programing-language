# tooling-dev — 工作状态
> 最后更新: 2026-09-27 17:30 by tooling-dev（P4.4：裸文件调用 + release 优化 + 打包/安装脚本）

## 当前状态
**P4.4 已完成**：`lfz <file>` 裸文件调用可用（**像 Python 那样 `lfz Hello.lfz` 直接运行**）；
release 构建加 `lto/codegen-units/strip`；新增 `scripts/build-release.ps1`（一键打包 `dist\lfz.exe`）
与 `scripts/install-lfz.ps1`（用户级安装，**默认 dry-run**，本次仅 dry-run 验证）。

### 本批交付（唯一改动范围）
- `src/cli.rs`：新增 `Command::Script`（裸文件调用）；`parse_args` 非 `-` 开头首参 = 脚本路径
  （恰一个）；`run_file` 抽出共用 `report_eval`，新 `run_script` 复用；新 `is_lfz_path`。
  裸文件入口收紧：须 `.lfz`（大小写不敏感），否则 `IOError: 只支持 .lfz 脚本文件：'<path>'`（exit 2）；
  `.lfz` 不存在 → `IOError: 无法读取：<path>`（exit 2）。`--help` 把 `lfz <file>` 置首；
  `--json` 位置不限（前置/后置/`run` 前后均可）。
- `Cargo.toml`：`[profile.release]` = `lto=true` / `codegen-units=1` / `strip=true`。
- `scripts/build-release.ps1`（新，**纯 ASCII** 3047 B）：`cargo build --release` → `dist\lfz.exe` →
  打印 `--version` → 冒烟 `examples\hello.lfz`；`-OutDir` 支持；失败非零退出。
- `scripts/install-lfz.ps1`（新，**纯 ASCII** 7354 B）：装到 `%LOCALAPPDATA%\Programs\lfz\`；
  `[Environment]::SetEnvironmentVariable('Path',...,'User')`（**非 setx**）追加 PATH；改前备份用户 PATH；
  `-Uninstall`；**默认 dry-run**，`-Apply` 才改（本次未跑 `-Apply`）。
- `tests/cli.rs` +8 e2e；`src/cli.rs` +5 单测（并有意更新 1 处旧断言：`frobnicate` 由「未知命令」
  改为裸文件路径——CLI 契约变更，已记 ADR）。

## 进行中
- （无；本批任务已交付，等待调度）

## 阻塞 / 需要支持
- （无）
- **需 team-lead 定夺**：① README（release-manager）与 `docs/guide/README.md`（docs-writer）需加
  `lfz <file>` 用法——建议文本已随汇报给出；② 用户 PATH 的**实际**修改：`install-lfz.ps1 -Apply`
  须经用户批准后由 team-lead 执行（本任务明令禁用 `-Apply`）。
- 历史待仲裁项（沿用）：`test_runner` 汇总 schema 为 spec 外扩展、`FAIL`/`ERROR` 并存取 `2` 等，
  见 ADR 2026-09-27 11:23；runner 契约 v1.2（§9.3 已解决）。

## 下一步计划（P4 剩余）
1. REPL（可选、非阻塞）。
2. `scripts/package.ps1`（8 项提交物打包，先向 team-lead 确认清单）+ `scripts/lfz.ps1`/`lfz.bat` 启动器。
3. 运行章节与 docs-writer 协作（`lfz <file>` / `lfz run` / `lfz test` / `--json` 用法 + 跨平台注意）。

## 关键经验（写给未来的自己）
- **裸文件 vs `run` 的边界**：裸 `lfz <file>` 收紧（须 `.lfz`）；`lfz run <file>` 保持旧行为
  （允许非 `.lfz`，非 `.lfz` 无 `#42` 要求且能正常跑）。两台形态的 `--json` 都走同一 `report_eval`，
  故 `--json` 下 stdout 恒为唯一合法 JSON；非 `.lfz` 用「与解释器错误同源」的 `LzError::Io` 渲染。
- **`parse_args` 语义变更**：`lfz frobnicate` 不再是「未知命令」，而是脚本路径（随后因非 `.lfz` 报错）。
  凡依赖旧「未知命令」文案的调用方需知悉；`-` 开头者仍是未知选项。
- **PowerShell 5.1 纯 ASCII 坑**：无 BOM 的 `.ps1` 按 ANSI 解码，**任何非 ASCII 字节都会解析崩**。
  两个新脚本非 ASCII 字节 = 0（用 `[IO.File]::ReadAllBytes` 计数 + `Parser::ParseFile` 双重确认）。
- **改用户 PATH 的正确姿势**：`[Environment]::SetEnvironmentVariable('Path', <new>, 'User')`，
  **禁用 `setx`**（会截断/合并）；改前备份；默认 dry-run，`-Apply` 才动。
- **release 体积**：`lto+codegen-units=1+strip` 后 `dist\lfz.exe` = **704000 B**（约 687 KiB）。
- **测试隔离**：单测与 e2e 依旧全部用系统临时目录（`tests/` 下只留 `*.rs`）。
- **控制台不渲染 CJK（证据采集）**：含中文的 `lfz test` 汇总 / `--help` 用 `Start-Process
  -RedirectStandardOutput <file>` 落原始字节，再用 Read 工具查看。
