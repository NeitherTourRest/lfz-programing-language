# runtime-dev 状态

2026-11-18 T27 join 候选已交主调度对照构建。src/builtins.rs SHA256 111acf0a4b80d1c094b5c95be49f59ebce1bd37bb161a3cd8091dee85f78e6df。与仅含 findIndex 的已冻结源码对照，唯一可执行差异是 join 借用原数组代替 Vec 克隆，另校正公开内置名单注释。逐字节逆向比较通过。

既有 Rust 单元测试和新黑盒已覆盖空数组、单元素、多元素、Unicode、首个非法元素及自引用数组诊断。当前静态检查完成，运行验证由主调度执行。
