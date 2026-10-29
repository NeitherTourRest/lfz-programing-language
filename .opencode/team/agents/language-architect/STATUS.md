# language-architect — 工作状态
> 最后更新：2026-10-29 by language-architect

## 当前状态
T16-B设计草案已完成并交team-lead，等待A独立通过后批准冻结；A既有§19完全未改。A仅身份/定位基础，不启用模块语法/文件加载/globals切换或CLI展示，不宣称产品或性能验收完成，R304仍OPEN。

## 产出与关键决定
- source.rs：SourceId/ModuleId不同newtype各ENTRY，SourceRecord loaded可None；SourceMap私有append-only Vec/new/get/checked insert；LocatedFailure持原错误Box与source ID。
- SourceMetadataError明确编号/帧数/未知来源/多余或缺失错误来源；attach_origins完整校验后原子替换，失败不动旧载荷，无panic或错误文件fallback。
- TracedRun私有可选origins，旧入口None；frame_source(index)/error_source安全Option，宿主后改公开frames/result的边界已说明。
- UserFn home_module整数，Interp active_module ENTRY，A所有实际函数均ENTRY，不强Rc持globals；公开UserFn literal新增字段属于Rust构造迁移，不能称无API变化。
- Span/LzError/TraceFrame/Loaded/Program保持旧形状；T15 Result三/五/三字段不加file，attempt不读sidecar；源表不持AST/Env/用户值，不扩大Transfer假设。
- core唯一source.rs/lib.rs；runtime唯一value.rs/evaluator.rs；tooling本批只读。B/C/D仍需分别冻结。

## 证据
docs/reports/T16-A-design.md 6730B；git diff --check -- docs/spec docs/reports/T16-A-design.md exit0（仅CRLF归一提示）。唯一§19标题检查、核心签名/owner/compat结构人工核对通过。ADR在写spec前只追加，本角色未动源/测试/看板或Git。

## 下一步/阻塞
core/runtime按§19实施与新增unit；独立复验旧549及公开回归。后继模块图/资源上限/缓存与home globals切换/CLI跨文件定位按B/C/D再冻结。无设计阻塞，不把基础元数据测试当模块执行证明。

## 关键经验
Loaded必须可缺失，读取/编码失败仍有来源身份；不能伪造入口文本。实际trace帧下标与func_id不同，来源必须逐帧记录。公开可变frames/result被宿主修改后，早先附加校验不构成永久保证。

## T16-B草案（2026-10-29）
docs/reports/T16-B-design-draft.md15785B；仅设计不改spec/source。明确ImportDecl/ExportDecl字段与export上下文兼容；保留Reserved import/from token；ModuleGraph拥有AST而SourceMap只metadata；迭代DFS，候选1024节点/128层含root/2MiB单文件/16MiB总raw bytes；root只有确认模块声明后启用预算，legacy大root不收紧，依赖limit+1限读。run/旧eval所有模块声明在任何副作用前整Program guard拒绝，B只check不初始化。建议RawSourceStats外层计数不改Loaded，GraphFailure带真实source/Loaded不得入口fallback。错误消息/限额均待B批准冻结；T15Result不加file。
证据：文件存在/长度、文档diff检查exit0；无构建/运行产品/Git/其他产物改动。下一步A独立验收后定案B。
