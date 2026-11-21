# release-manager — 工作状态
> 最后更新：2026-11-20 by release-manager

T29 暂存准备脚本已生成于 Temp/T29/prepare-checkpoint.py，基线66565e31bf677855b79502f15c47fc729466ebe3。白名单只允许env/evaluator、screen.py、results、T29-independent证据、T29-callback-frames报告与参与角色/团队记录。

准备时须指定 --decision retain-source 或 archive-only。保留源码模式核两源码与冻结ZIP SHA一致；仅归档模式要求工作源码先由root恢复到基线，ZIP与清单仍逐成员精确核对。脚本不自动恢复源、不自动切换分支。--stage-group沿T28逐项验证工作字节/副本/index blob。三保护块28/1/1过滤后等于HEAD时不入expected stage。main和dist身份保留检查。

已实跑 retain-source 准备模式成功，当前四组含空独立组（最终证据落盘后刷新）。未执行Git写入或启动LFZ/Cargo，筛选最终结论等待主调度。

T29 benchmark-inputs.zip及manifest已落盘并回读通过，11仓库相对路径，25746B/SHA8800480693667d9c296412564fafea3d3ada488a6749dcac9ac698fa57d9383a。各预声明/协议SHA全部匹配，入口额外原件SHA限制记入notes。无Git/LFZ/计时执行。

2026-11-21 交付准备已齐：Temp/T29/prepare-delivery.py 默认只读，已核两ZIP全部成员及新旧实物；Temp/T29/delivery-steps.md 描述候选四commit后GO第五交付提交步骤。dist仍为T26，README/交付清单待明确GO。
