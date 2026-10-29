# T16-A 来源与模块身份基础设计

日期：2026-10-29。责任人language-architect。经team-lead批准，三spec §19已冻结；本文是接口交接，不是实现验收报告。
实际核查：TracedRun已有private func_names，全仓仅eval_module_on_thread_with_args一处构造；UserFn仅make_closure一处构造。Loaded仅text/line_base且可Clone，现CLI统一使用入口path。保留旧形状，避免全链错误literal迁移。

## 19. T16-A 来源身份与定义模块基础（2026-10-29 冻结）

### 19.1 范围与责任

本节仅为模块前置基础；A不解析import/export、不加载模块/IO、不切globals、不改CLI输出、不新增内置。Span、LzError、TraceFrame、Loaded、Program旧形状及T15 Result/error/trace三/五/三字段保持。73内置、旧入口及旧无模块程序逐字节兼容。core-dev唯一写src/source.rs及src/lib.rs声明；runtime-dev唯一写src/value.rs、src/evaluator.rs；tooling本批只读。B/C/D另冻结，不以本批冒称模块可用。

### 19.2 source.rs 精确公开签名

新增不同newtype SourceId(pub u32)、ModuleId(pub u32)，均derive Clone/Copy/Debug/PartialEq/Eq/Hash，各自impl关联常量ENTRY=Self(0)。两编号分域，不可按相同整数推断同一身份，ModuleId只在所属session有意义。

SourceRecord derive Clone/Debug，公开字段：
- identity: Option<std::path::PathBuf>
- display_path: String
- loaded: Option<crate::loader::Loaded>

SourceMap derive Clone/Debug/Default，私有Vec<SourceRecord>，公开方法：
- pub fn new() -> Self
- pub fn len(&self) -> usize
- pub fn is_empty(&self) -> bool
- pub fn get(&self, id: SourceId) -> Option<&SourceRecord>
- pub fn insert(&mut self, record: SourceRecord) -> Result<SourceId, SourceMetadataError>

SourceMetadataError derive Clone/Debug/PartialEq/Eq，公开enum变体：
- TooManySources
- FrameCount { expected: usize, actual: usize }
- UnknownSource { id: SourceId }
- UnexpectedErrorSource
- MissingErrorSource

LocatedFailure derive Debug，公开字段source_id: SourceId、error: Box<crate::error::LzError>。只关联既有错误与来源，不改语言错误类/消息。ID在拥有SourceMap的边界验证，不是跨map全局身份。

insert先checked转换当前len为u32，失败返回TooManySources且不修改map；首次成功ID为ENTRY，后续按插入序稳定递增，无删除/重排。禁止截断cast；此检查不宣称捕获宿主OOM。get未知ID为None，不panic、不回退ENTRY。测试编号上限用有界辅助逻辑，禁止为测试分配u32::MAX个记录。

SourceMap只持元数据/可选Loaded，不持Program、Env、Rc用户值或求值器；不扩大现Transfer跨线程假设。A不canonicalize/读文件/去重/小写化路径，identity只是调用者提供身份，display_path允许虚拟路径。别名合并和路径验证归后继图层。加载失败允许loaded=None，禁止伪造入口文本给该错误画下划线。A不提供文本渲染。

### 19.3 运行时扩展签名与验证

UserFn新增公开home_module: crate::source::ModuleId。make_closure记录Interp私有active_module，Interp::new初始化active_module=ModuleId::ENTRY；A全部实际执行仅ENTRY，不注册/切换模块，不改查名、globals父链、self、cell、func_id和函数ID表。只存整数不Rc持home Env，避免普通模块必然引用环。没有新增跨session函数值调用API。

UserFn公开结构新增字段属于Rust构造API迁移：仓库唯一make_closure构造点机械补字段，外部手写literal须补home_module，不能声称完全无Rust API变化。TracedRun本已有私有func_names，外部原本不能literal构造；仓库唯一构造点补新私有origins=None。

TracedRun私有origins: Option<RunOrigins>，RunOrigins由runtime私有实现，持Rc<SourceMap>、Vec<SourceId>逐帧来源及Option<SourceId>错误来源。旧eval_module*入口全部origins=None，不自动生成来源表；CLI仍旧路径不读新sidecar。

新增TracedRun公开方法：
- pub fn attach_origins(&mut self, sources: std::rc::Rc<crate::source::SourceMap>, frame_sources: Vec<crate::source::SourceId>, error_source: Option<crate::source::SourceId>) -> Result<(), crate::source::SourceMetadataError>
- pub fn frame_source(&self, index: usize) -> Option<&crate::source::SourceRecord>
- pub fn error_source(&self) -> Option<&crate::source::SourceRecord>

attach_origins先完整校验再原子替换；任一失败保留旧sidecar。校验顺序规范性：1)frame_sources.len与self.frames.len相同，否则FrameCount{expected:self.frames.len(),actual:frame_sources.len()}；2)按帧序每ID须存在，否则UnknownSource；3)结果Ok时error_source须None，否则UnexpectedErrorSource；结果Err须Some，否则MissingErrorSource；4)存在的error_source也须有效，否则UnknownSource。返回结构化宿主元数据错误，不panic、不伪装LzError、不把坏来源悄悄改为入口文件。

frame_source用实际帧下标，不是func_id；无sidecar/越界/宿主后改公开frames导致无法匹配时None。error_source在无sidecar或当前结果非Err时None；查询每次经SourceMap::get安全访问。宿主后改公开frames/result可能使旧attach保证失效，访问器仍须安全。后继诊断调用者必须显式处理None，不能拿其它文件代替出错文件。

A只附加载荷，不根据active状态在冒泡后猜真实错误source；后继B/C/D在错误发生/压帧时记source再附加。T15 attempt继续只序列化旧TraceFrame固定字段，完全忽略origins；A active_module始终ENTRY，成功/恢复后仍ENTRY。真正模块切换与恢复快照扩展归C，不在A先轮换globals。

### 19.4 最低验收及后继

core新增unit覆盖空map/插入稳定ID/loaded=None/未知ID/虚拟路径/LocatedFailure保留错误与位置。runtime新增unit覆盖legacy None、成功与错误附加、不同帧不同来源、所有拒绝变体与原子不修改、越界None、home_module ENTRY及attempt恢复后ENTRY/旧trace保持。允许合成多来源元数据，不能把它当模块执行证据。旧549测试及CLI/check/attempt/JSON/app行为继续通过；新计时前不得宣称性能改善，R304仍OPEN。

B另冻结ImportDecl/ExportDecl、模块图及受控上限；C另冻结session缓存/独立globals/home切换；D接真实跨文件CLI诊断。不得预拼源码、隐式global注入或多个Interp互调绕过身份约束。
## 证据与限制
只读核查src/loader.rs,error.rs,value.rs,evaluator.rs,cli.rs,lib.rs及全仓literal。只修改三spec、本报告、追加ADR与自身记录；无源/测试/看板/Git写入。B/C/D继续独立冻结，T16草案不作为A已实现能力，R304仍OPEN。
