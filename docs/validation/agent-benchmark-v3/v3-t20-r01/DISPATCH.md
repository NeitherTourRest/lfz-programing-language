# 主调度交付与派发

独立控制签收后，另存 `manifest.frozen.json` 并设 `status` 为 `FROZEN`。`independent_review` 填实际审核结果路径、SHA 与范围，`final_binary` 填项目相对路径、字节数和 SHA256，`final_ai_docs` 填五文档路径和 SHA256。核全部 `files` 字节，保存原始冻结 ZIP。草稿只供审阅。

按题准备工作区，立即派发并记录真实 UTC、session ID、agent path 和派发提示 SHA。预算从 `cycle-config.json` 记录时点开始。保持各题一致的实际工具与模型条件。

```powershell
python -B docs/validation/agent-benchmark-v3/v3-t20-r01/stage_student.py --manifest docs/validation/agent-benchmark-v3/v3-t20-r01/manifest.frozen.json --task C01 --run-id v3-t20-r01
```

派发文本模板如下。只替换绝对工作区路径、允许的 Python 命令及 C04 公开服务地址。派发使用无历史会话。题面所在目录是允许读取的工作区。C04 使用预先启动的公开 loopback 服务或题面允许的自调方式，记录服务起止与统一 OS 许可。

> 在 WORKSPACE 中完成 TASK.md。读取该目录内的 LFZ 指南及公开输入。只在此目录写入提交和自调文件。用 `PYTHON -B student_cycle.py --config cycle-config.json --argv-json JSON_ARRAY` 自调。按题目需要可用 `--max-steps 100`、`--json`，C04 可用 `--allow-http`。每周期由工具先 check 后 run，最多五周期。把提交文件及实际命令记录在本工作区。请在配置的截止时间前交卷。

封存交卷源码和自调记录，主调度保存真实会话工具调用与返回。独立审计按 AUDIT_PROTOCOL.md 对照记录，缺完整工具证据保留 UNKNOWN。以 runner.py 对封存提交评分。学生不可见私有目录、控制件、oracle、验收记录或其它题。
