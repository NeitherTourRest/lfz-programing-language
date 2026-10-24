# 源码约束独立审查

verifier，仅静态核对封存源码并结合冻结功能结果；未修改学生答案、未给学生反馈。

| 题 | 源码约束 | 判定 |
|---|---|---|
| B01 | solution.lfz:3-83，自行递归下降expr/term/unary；ord逐位int累积，div向下整除，二元循环左结合、括号递归；无文本答案表 | PASS |
| B02 | solution.lfz:2-65，字符状态机quoted，双引号转义、换行/CRLF与末尾字段状态；预分配数组后逐格填充；不是示例硬编码 | PASS |
| B03 | solution.lfz:2-7码点逐标量求和mod8；11行显式8个独立桶；24-29完整字符串key比较；31-50插入/覆盖/删除/计数；每次solve局部初始化；不用动态struct冒充哈希表 | PASS |
| B04 | solution.lfz:11-12方向上右下左；17-35 head/tail FIFO且31行首次发现即前驱标记；38-55无路判定/回溯；board为字符拷贝，无grid/start/goal写操作 | PASS |
| B05 | solution.lfz:5-27二维DP按原物品顺序；20行严格大于才选，平局不选当前；30-38从末项回溯并升序filter，局部初始化 | PASS |
| B06 | solution.lfz:3局部空账本；9-22建立所有提及账户；23-29先判足额，再扣加，自转账足额不变；无拒绝时部分扣款 | PASS |
| B07 | solution.lfz:12-30原文ASCII码范围分词后lower；31-42预分配并一次构造倒排位置；45-54仅查既有索引，不逐query重扫text | PASS |
| B08 | solution.lfz:3真实readFile，9-14 trim/int并累计，17真实writeFile一次覆盖，UTF8固定LF字符串；不写inputPath，无顶层IO | PASS |

八份学生solution仅函数定义及注释，无顶层input、示例文件或提前退出；源码中无网络/外部程序/调用其他语言，未见固定验收答案。此结论只涉及封存源码内容，不能证明开发期间未作未记录外部访问。功能结果独立131/131通过；源码约束PASS不替代协议审查。