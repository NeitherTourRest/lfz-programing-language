# release-manager — 工作状态
> 最后更新: 2026-09-23 by release-manager
## 当前状态
P0.5 **本地版本基线已完成**：git 仓库建立（`main`）+ 初始提交 `e060b21` + 附注标签 `v0.1.0` + 版本管理纪律 ADR。**远程 GitHub 仓库与推送**待 team-lead 与用户确认参数后另派（本次未做任何远程操作）。
## 进行中
- （无；P0.5 本地部分已交付，等待 team-lead 验收与远程推进指令）
## 阻塞 / 需要支持
- **远程推送阻塞于用户确认**：仓库名 / 可见性（public/private）/ 认证方式（HTTPS+PAT vs SSH）。确认前严禁任何远程操作（不装 `gh`、不 `remote add`、不 push）。
## 下一步计划
- 待 team-lead 授权后执行 `git remote add origin <url>` + 首次 `git push -u origin main`（参数由用户确认）。
- 后续每个阶段里程碑：由我统一做**原子提交** + **附注标签**，并**实时更新 README**。
- P10：产出交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **P0.5 基线事实**：初始提交 `e060b21`（67 文件，11627 插入）；标签 `v0.1.0`；默认分支 `main`；`git ls-files` = 67。
- **含非 ASCII 提交信息**：em-dash（—）等字符写成 UTF-8 临时文件（`C:\Users\19170\AppData\Local\Temp\opencode\lfz-commit-msg.txt`）后用 `git commit -F <file>`，避免 PowerShell 控制台编码损坏。
- **Windows 行尾**：本机 `core.autocrlf=true`，`git add` 打印 "LF will be replaced by CRLF" 警告（无害）；仓库内存 LF，工作区状态保持干净。
- **`.gitignore` 边界纪律**：`.opencode/` 必须入库、`.omo/` + `.codegraph/` 忽略、`Cargo.lock` 入库（二进制应用）；已新增 Rust 条目 `/target/`、`**/*.rs.bk`、`*.pdb`。
- **目录实况**：`docs/spec/` = 3 文件（v1 三件套）；`.opencode/team/` 另含 BRAINSTORM / DRAFT-LFZ-v0.2~v0.5 / v1 / DRAFT-runtime-arch / KICKOFF 等历史文档，均已入库。
- **禁区确认**：本次全程零远程操作。
