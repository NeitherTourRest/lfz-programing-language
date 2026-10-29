# T16-A 运行时身份与来源sidecar作者验证

日期：2026-10-29。依据三spec §19及T16-A-design。只为后继模块基础，不实现import/export、模块图、模块初始化或跨文件CLI输出。

## 实现

UserFn新增整数home_module，make_closure取Interp.active_module；旧构造从ModuleId::ENTRY开始，A仅ENTRY，不切globals，不强Rc持模块Env。UserFn公开literal需新增字段，是明确Rust构造API迁移，不宣称Rust API完全无变化。原语言查名/cell/self/函数ID行为未变。

TracedRun新增私有可选RunOrigins。旧入口origins=None。attach_origins按帧数、帧序来源ID、成功/错误来源要求、错误ID顺序完整校验，最后才替换；拒绝保留旧sidecar，宿主元数据错误不冒充语言异常。frame_source按实际帧下标，缺载荷/越界/宿主改帧数不匹配返回None；error_source仅当前Err且合法载荷返回来源，不回退入口。查询不渲染文本，不修改旧CLI。

Span/LzError/TraceFrame/Loaded/Program保持旧形状；T15错误数据固定字段未增加file，不读取新sidecar。core独立完成source.rs/lib.rs，runtime只改value.rs/evaluator.rs。

## 验证

新增7项runtime测试：旧求值入口来源None；命名/匿名/逃逸闭包home为ENTRY；不同帧不同来源、合法替换；各拒绝变体和规范验证次序保持旧sidecar；成功禁止错误来源且先于错误ID查找；宿主修改公开frames/result仍安全、loaded=None；attempt恢复后active ENTRY及原trace/depth、旧Result/error/frame字段精确不变。

`cargo test --release t16_a_ --lib`：7/7。之后完整`cargo test --release`：562/562（432lib+48bin+28cli+12cli_t12+12cli_t14+15runner+15unit），0失败/忽略，其中core另增6项。`cargo build --release`成功，最后no-op。两个修改源`git diff --check`通过，旧测试未删改。未声称本轮新增测试在旧实现先红；它们验证新增宿主元数据接口与保持不变的语言边界。

产物target/release/lfz.exe 783872B，SHA256 `65a29eaea54f32d1d2b404f22aed09389c3ffa71e6a81acb8fa6351d32fa42b7`。原始日志及完整source/Cargo/二进制指纹位于T16-A-runtime-evidence，按原字节保存。工具链和TEMP均项目内，只release、复用target，编译由team-lead串行协调，未改dist/Cargo/profile/Git。环境canonicalize警告保留，不声称零warning。

## 交接

源码与候选冻结，作者不再编译/改源。独立验收、原CLI/attempt/应用差分及发布复制由team-lead后续调度。来源载荷测试使用合成元数据，不作为真实模块执行证据。R304保持OPEN，未性能计时或测RSS，不据此批准正式发布。B/C/D须另冻结、实现和验收。
