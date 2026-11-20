# release-manager — 工作状态
> 最后更新：2026-11-20 by release-manager

T28 精确提交脚本已准备在 Temp/T28/prepare-checkpoint.py。当前分支 codex/t27-find-index，基线8148be0，提交四组：运行时、功能验收、性能原始记录、交付与团队记录。最终全员落盘后root再刷新一次清单，审查后逐组执行 --stage-group N。

原用户28/1/1块从Temp副本过滤，过滤后与HEAD同blob者从expected staged集合排除，仍验证工作区原字节。源env/evaluator与新增test使用冻结副本。每组暂存先验证工作字节与源副本，最终逐一对照index blob；main及dist 7a144213身份检查。raw .log保字节，diff-check按cr-at-eol,-blank-at-eof执行。

性能和独立最终回执待root签收，未作晋级声明。本角色无Git写入、未运行LFZ/Cargo，未改产品、main、dist或旧冻结资料。

第3组已明确允许 benchmarks/t28/diagnose.py，用于 array_builtins 残余成本诊断。该文件本次检查尚未落盘，最终生成清单时按实际文件纳入。
