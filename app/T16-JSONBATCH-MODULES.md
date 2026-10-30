# T16 四模块JSONL批处理应用

日期：2026-10-30。app-dev。依据三spec §20/§21与T16-C设计；在LFZ skill基础上补读模块规范，不凭其它语言猜import/export。旧T15 jsonbatch与其oracle保持不变。

## 结构与使用

- `jsonbatch_modules/common.lfz`：共享state、mark计数和require校验；使用定义模块label构造错误。
- `record.lfz`：导入common，每解析一条先mark，再以attempt捕获jsonParse错误。
- `batch.lfz`：同时导入common和record，构成菱形依赖，按原行顺序构建结果。
- `main.lfz`：CLI入口，两参数INPUT OUTPUT；导入common与batch，先读/验证/处理/编码再写。

```powershell
./dist/lfz.exe check app/jsonbatch_modules/main.lfz
./dist/lfz.exe run app/jsonbatch_modules/main.lfz -- app/fixtures/t15/mixed.jsonl Temp/T16-app/result.json
```

文件模块路径相对导入文件，IO路径相对进程cwd；命令从仓库根运行，先创建项目Temp/T16-app输出目录。模块只用显式导出，无动态import或包依赖。当前candidate最终hash验收信息见本文件后续结果节，不能用B仅构图结果冒充运行结果。

## 可观察协议

沿用T15结果`total/passed/failed/rows`，另加`moduleCalls`。每个真实记录恰调用一次公共mark，正常输出moduleCalls=total。record和batch若各得到独立common实例，该值会错误地为0；这是实际共享模块实例的见证。连续两次独立进程要求输出字节一致且计数重置，不能遗留跨执行cache。

行策略与原应用一致：UTF8整文件读取，CRLF转LF，空文件0条，末尾一个LF只移除虚拟尾片段；真实空行不跳过，坏JSON逐条失败后继续，末行不要求换行。裸CR不独立分行。成功null与失败nil靠ok区分；error.class需索引`["class"]`，不是点关键字。error的line/col指LFZ源码调用位置，外层line才是输入物理行号；T15 attempt协议无file，不能宣传错误记录包含跨模块文件名。

common.label与main.label刻意不同；校验失败应来自common的jsonbatch-modules前缀，不能串到caller同名全局。错误后已有IO、容器修改不回滚。本例仅在输出前完成处理和编码，writeFile本身非原子；路径仅作字面相等检测，实际别名输入输出须调用者避免。

## 开发记录与验证准备

按team-lead任务将T15单文件逻辑拆为四文件，未修改旧应用。数据处理使用range/map/filter而非循环push；独立Python verifier复用旧T15 strict JSON oracle函数，不导入解释器实现，新增模块状态/重复进程/定义全局检查。

`verify_jsonbatch_modules.py` AST自检通过。准备33进程矩阵：完整图check1；13场景重复执行26（混合、空文件、单值、末尾LF、空行、连续空行、中间空行、CRLF、nil/false/0、Unicode、i64、重复键、连续80错误后成功）；另参数数目/空路径/同路径/缺文件/非法UTF8失败6。所有失败路径核对输入字节及已有输出不变。

```powershell
D:/app/ANACONDA/python.exe -B app/verify_jsonbatch_modules.py --lfz target/release/lfz.exe --expected-sha FINAL_SHA256 --output Temp/T16-app/final.json
```

当前仅准备，等待tooling修HostFailure runner后的最终candidatehash；未在旧/修前产物上宣称通过，未编译或Git。验证输出须是不存在的项目路径。图构建可能受宿主canonicalize权限影响，若出现须保留失败证据并报告，不自行规避。

本例仍整文件驻内存，不声称流式或性能改善；R304保持OPEN。模块图、来源与CLI全面边界由独立verifier验收，本应用不替代语言测试。

## 最终作者实跑（2026-10-30）

最终tooling已冻结C产物`target/release/lfz.exe`：902656B，SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`。核hash后默认沙箱首次在check图阶段拒绝common.lfz读取/canonicalize，exit2/IOError；[原始失败](t16-evidence/default-permission-failure.json)保留，未修改产品绕过。

同一精确产物获许可在项目内提权执行原验证命令，**33/33通过，exit0**；原始结果见[results.json](t16-evidence/results.json)，源与脚本/结果指纹见[manifest.json](t16-evidence/manifest.json)。提权请求31秒时曾通知root疑审批等待，随后正常返回；不是应用挂起或进程超时。没有编译、改源码/profile、复制大二进制或执行Git。

确认的实际行为：完整图check不执行main参数校验；13组输入含坏记录和Unicode均按独立JSON oracle输出；两个consumer经菱形导入看到同一state，moduleCalls=记录数；每组两次独立进程输出字节相同且计数从0重建，未串跨执行cache；参数校验错误前缀来自common.home全局而非entry同名label；6种错误路径保留输入和已有输出字节。33为本应用进程检查数，不能叠加成官方LFZ runner用例数。

本节是最终实测，前文“等待最终hash”保留为准备阶段记录。C修前671919与B构图候选结果都未冒充本应用最终验收。独立verifier仍需另行核验，作者通过不关闭性能R304或批准正式发布。
