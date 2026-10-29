# T16-A 独立验收准备
日期：2026-10-29。状态：仅准备，未编译、未启动解释器或执行候选验收。

已按§19读取精确接口并准备Temp/T16-A-verifier/witness.rs。最终批准编译后验证：来源ID稳定、未知ID安全None、Loaded原文与line_base不变、缺源码记录、多帧不同来源、错误元数据判定顺序、原子保留旧sidecar、legacy空sidecar、安全公开字段突变查询、UserFn home_module ENTRY。编号容量上限仅查core有界helper测试，不分配真实巨大注册表。

兼容流程：最终候选冻结后，直接运行新release harness并保留实际总数；复用T15 attempt13/T14 check117/T12 CLI91含官方178/旧JSON140/旧app37+11/jsonbatch38，结果绑定最终hash。额外新旧二进制使用完全相同脚本路径与argv，对stdout/stderr/exit逐字节比较，覆盖正常函数、恢复后错误、嵌套attempt、管道/argv、资源容量受控错误、source接口不改变CLI机器对象。

旧dist精确副本：Temp/T16-A-verifier/lfz-before.exe，782848B，SHA256 1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c。只复制并hash核验，未重建旧版。所有临时件项目内。

最终独立Rust witness只能在主调度准许、core/runtime编译结束之后使用现release rlib单次rustc编译。若断言失败，先比§19定位产品缺陷/作者oracle，保留首次失败，不修改产品。A不是import/export执行功能，无模块初始化或性能通过声明；R304仍开放。
