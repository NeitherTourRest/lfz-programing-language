//! 加载器：UTF-8 校验 / BOM / 行终止符归一 / `ext(path)` / `#42` 前导 / `line_base`。
//!
//! 单一事实源（本模块**只读消费**，不得偏离）：
//! - `docs/spec/interface-contract.md` §10.1（loader 双入口 + `ext(path)` + `Loaded{text,line_base}`）。
//! - `docs/spec/syntax.md` §2.1（编码 / BOM / 行终止符）、§2.2.0（`ext` 判定）、
//!   §2.2.1（前导定义，`CosmosAnswerError`）、§2.2.2（加载器逐字节算法，**规范性**）。
//! - `docs/spec/semantics.md` §8.1（`CosmosAnswerError` / `SyntaxError::NotUtf8` / `IOError` 消息）。
//!
//! 边界：本模块**只做加载**，不做词法 / 语法 / 求值；前导**不产出任何 token**。
//! 错误构造走 `crate::error` 的 `#[cold]` 构造器（热路径只走 `Ok`）。

use crate::error::{self, R, SyntaxMsg};
use crate::span::Span;

/// 加载结果（`interface-contract.md` §10.1）。
///
/// `text` = 归一化后的源码文本（已跳 BOM、行终止符统一为 `\n`；`.lfz` 文件已消费 `#42` 前导行）。
/// `line_base` = 行号基数：消费了前导 → `1`；否则 → `0`。
/// lexer 报告的 `line` = 本地行号 + `line_base`（`syntax.md` §2.2.2 行号语义）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Loaded {
    /// 归一化后的源码文本。
    pub text: String,
    /// 行号基数（`syntax.md` §2.2.2）：`.lfz` 消费前导 → 1；否则 → 0。
    pub line_base: u32,
}

/// 取路径的扩展名（`interface-contract.md` §10.1 / `syntax.md` §2.2.0 步骤 1–3）。
///
/// 步骤 1：按 `'/'` 与 `'\'` 切分，取最后一段路径分量（`'C:'` 的 `':'` 不参与切分）。
/// 步骤 2：该段不含 `'.'` → `None`（无扩展名）。
/// 步骤 3：否则取**最后一个** `'.'` 之后的子串（**可为空串**，如 `"a."`）。
///
/// 步骤 4（与 `"lfz"` 的 ASCII 大小写不敏感比较）见 [`is_lfz`]。
/// **只看路径字符串**：不访问文件系统、不解析符号链接 / 8.3 短名。
#[must_use]
pub fn ext(path: &str) -> Option<&str> {
    // 步骤 1：按 '/' 与 '\' 切分取最后一段（'C:' 的 ':' 不参与切分）。
    let last = path.rsplit(|c: char| c == '/' || c == '\\').next().unwrap_or("");
    // 步骤 2：不含 '.' → 无扩展名。
    let dot = last.rfind('.')?;
    // 步骤 3：最后一个 '.' 之后的子串（可为空串）。
    Some(&last[dot + 1..])
}

/// `ext(path)`（`syntax.md` §2.2.0 步骤 4）：扩展名是否等于 `"lfz"`（**仅 ASCII** 大小写不敏感）。
///
/// `foo.lfz` / `Foo.LFZ` / `a.b.Lfz` → `true`；`foo.txt` / `foo` / `foo.` / `dir.lfz/x` → `false`。
#[must_use]
pub fn is_lfz(path: &str) -> bool {
    // 步骤 4：与 "lfz" 比较时按 ASCII 大小写不敏感。
    ext(path).is_some_and(|e| e.eq_ignore_ascii_case("lfz"))
}

/// `load_file(path)`（`syntax.md` §2.2.2，**规范性**）。
///
/// 读文件 → UTF-8 校验 → 跳 BOM → 归一化行终止符 → 按扩展名分流：
/// `.lfz` **要求并消费** `#42` 前导（`line_base = 1`）；否则与 [`load_source`] 同等（`line_base = 0`）。
///
/// 错误：读取失败 → `IOError`（`无法读取：{path}`，无位置）；非 UTF-8 → `SyntaxError`
/// （`NotUtf8 { off }`，**先编码、后前导**）；`.lfz` 前导违规 → `CosmosAnswerError`。
pub fn load_file(path: &str) -> R<Loaded> {
    // 2. 读入全部字节；失败 → IOError（无源码位置）。
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Err(error::io(format!("无法读取：{path}"), None)),
    };
    // 3. 按 UTF-8 解码整文件（先编码、后前导）；失败 → SyntaxError。
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(err) => {
            let off = err.utf8_error().valid_up_to();
            return Err(error::syntax(SyntaxMsg::NotUtf8 { off }, Span::START));
        }
    };
    // 4–5. 跳 BOM + 行终止符归一化。
    let text = normalize(text);
    // 6–7. 按扩展名分流：`.lfz` 要求并消费前导，否则不校验。
    if is_lfz(path) {
        consume_preamble(text)
    } else {
        Ok(Loaded { text, line_base: 0 })
    }
}

/// `load_source(text)`（`syntax.md` §2.2.2，REPL / stdin / `-e`）。
///
/// **不要求、不消费**前导；仍跳 BOM 并归一化行终止符；`line_base = 0`。
pub fn load_source(text: &str) -> R<Loaded> {
    // 仍做跳 BOM + 归一化（§2.2.2 load_source）；不要求、不消费前导；line_base = 0。
    Ok(Loaded {
        text: normalize(text.to_string()),
        line_base: 0,
    })
}

/// 跳一个 BOM（`U+FEFF`，仅开头、仅一次）+ 行终止符归一化（`syntax.md` §2.1 / §2.2.2 步骤 4–5）。
fn normalize(text: String) -> String {
    let mut text = text;
    // 步骤 4：仅当以 BOM 开头时去掉一个该字符（双 BOM 只去第一个）。
    if text.starts_with('\u{FEFF}') {
        text.replace_range(..'\u{FEFF}'.len_utf8(), "");
    }
    // 步骤 5：`\r\n` 与单独 `\r` → `\n`（先处理两字符序列，再处理裸 `\r`）。
    if text.contains('\r') {
        text = text.replace("\r\n", "\n").replace('\r', "\n");
    }
    text
}

/// `.lfz` 前导校验与消费（`syntax.md` §2.2.2 步骤 7a–e）。
///
/// 前导 = 第 1 行**恰好** 3 字符 `#42`，**其后紧跟**（已归一化的）`\n`；
/// 空文件 / 仅 3 字节无换行 / 尾随空格或 Tab / 大小写或字符不符 → `CosmosAnswerError`。
/// 消费整个前导行后 `line_base = 1`（程序体自 line 2 起）。
fn consume_preamble(mut text: String) -> R<Loaded> {
    // 7a–c：前 4 字节必须恰为 `#42` + 行终止符（已归一化为 `\n`）。
    if !text.as_bytes().starts_with(b"#42\n") {
        return Err(error::cosmos_answer());
    }
    // 7d：消费 "#42" 三字符与其后的 '\n'（整个前导行，不产出 token）。
    Ok(Loaded {
        text: text.split_off(4),
        line_base: 1,
    })
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 测试用临时文件：创建于系统临时目录，`Drop` 时删除（不在项目目录留残留）。
    struct TempFile(PathBuf);

    impl TempFile {
        fn new(ext: &str, bytes: &[u8]) -> Self {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos());
            let stem = format!("lfz_loader_{}_{}_{}", std::process::id(), nanos, n);
            let name = if ext.is_empty() {
                stem
            } else {
                format!("{stem}.{ext}")
            };
            let path = std::env::temp_dir().join(name);
            std::fs::write(&path, bytes).expect("写临时文件失败");
            TempFile(path)
        }

        fn path(&self) -> String {
            self.0.to_str().expect("临时路径应为 UTF-8").to_string()
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    // ---- ext / is_lfz 矩阵（§10.1 步骤 1–4）----

    #[test]
    fn ext_returns_last_component_extension() {
        assert_eq!(ext("a.lfz"), Some("lfz"));
        assert_eq!(ext("a.LFZ"), Some("LFZ"));
        assert_eq!(ext("a.Lfz"), Some("Lfz"));
        assert_eq!(ext("a.txt"), Some("txt"));
        assert_eq!(ext("a"), None, "无 '.' → 无扩展名");
        assert_eq!(ext("a."), Some(""), "最后一个 '.' 之后为空串");
        assert_eq!(ext("dir.lfz/x"), None, "只看最后一段分量 'x'");
        assert_eq!(ext("dir/sub/a.lfz"), Some("lfz"));
        assert_eq!(ext(r"C:\p\a.lfz"), Some("lfz"), "'C:' 的 ':' 不参与切分");
        assert_eq!(ext("a.lfz."), Some(""), "取最后一个 '.'");
        assert_eq!(ext(".gitignore"), Some("gitignore"));
        assert_eq!(ext("foo.lfz.bak"), Some("bak"));
        assert_eq!(ext("C:"), None);
        assert_eq!(ext(""), None);
    }

    #[test]
    fn is_lfz_is_ascii_case_insensitive() {
        assert!(is_lfz("a.lfz"));
        assert!(is_lfz("a.LFZ"));
        assert!(is_lfz("a.Lfz"));
        assert!(is_lfz("a.lfZ"));
        assert!(is_lfz("dir/sub/a.lfz"));
        assert!(is_lfz(r"C:\p\a.lfz"));
        assert!(!is_lfz("a.txt"));
        assert!(!is_lfz("a"));
        assert!(!is_lfz("a."));
        assert!(!is_lfz("dir.lfz/x"));
        assert!(!is_lfz("foo.lfz.bak"));
        assert!(!is_lfz("foo.lfz.txt"));
        assert!(!is_lfz("a.lfz."));
        assert!(!is_lfz("a.ｌｆｚ"), "仅 ASCII 折叠，全角不匹配");
    }

    // ---- BOM ----

    #[test]
    fn bom_is_skipped_for_file_and_source() {
        let f = TempFile::new("lfz", b"\xEF\xBB\xBF#42\n");
        let l = load_file(&f.path()).expect("带 BOM 的 .lfz 应合法");
        assert_eq!(l.text, "");
        assert_eq!(l.line_base, 1);

        let l = load_source("\u{FEFF}abc").expect("load_source 恒 Ok");
        assert_eq!(l.text, "abc");
        assert_eq!(l.line_base, 0);

        let l = load_source("\u{FEFF}\u{FEFF}x").expect("load_source 恒 Ok");
        assert_eq!(l.text, "\u{FEFF}x", "仅跳过一个 BOM");
    }

    // ---- 行终止符归一化 ----

    #[test]
    fn line_terminators_normalize_to_lf() {
        assert_eq!(load_source("a\r\nb\rc\nd").unwrap().text, "a\nb\nc\nd");
        assert_eq!(load_source("x\ry").unwrap().text, "x\ny");
        assert_eq!(load_source("keep\nlf").unwrap().text, "keep\nlf");

        let lf = TempFile::new("lfz", b"#42\nlet x = 1\n");
        let crlf = TempFile::new("lfz", b"#42\r\nlet x = 1\r\n");
        let cr = TempFile::new("lfz", b"#42\rlet x = 1\r");
        let a = load_file(&lf.path()).unwrap();
        let b = load_file(&crlf.path()).unwrap();
        let c = load_file(&cr.path()).unwrap();
        assert_eq!(a, b, "LF 与 CRLF 归一后等价");
        assert_eq!(b, c, "CRLF 与 CR 归一后等价");
        assert_eq!(a.text, "let x = 1\n");
        assert_eq!(a.line_base, 1);
    }

    // ---- `#42` 前导：违规全部 → CosmosAnswerError ----

    #[test]
    fn missing_or_malformed_preamble_is_cosmos_answer() {
        let cases: &[&[u8]] = &[
            b"",                       // 空文件
            b"#42",                    // 无行终止符（EOF）
            b"#43\n",                  // 字符不符
            b"#42abc\n",               // 多余字符
            b"#42 \n",                 // 尾随空格
            b"#42\t\n",                // Tab
            b"# 42\n",                 // 空格
            b"##42\n",                 // 双 '#'
            b"\n#42\n",                // 前置空行
            b" #42\n",                 // 前置空格
            b"\t#42\n",                // 前置 Tab
            b"#42;\n",                 // 尾随分号
            b"#42//x\n",               // 尾随注释
            b"\xEF\xBB\xBF",           // 仅 BOM
            b"\xEF\xBB\xBF\n#42\n",    // BOM 后有换行
            b"\xEF\xBB\xBF\xEF\xBB\xBF#42\n", // 双 BOM
        ];
        for bytes in cases {
            let f = TempFile::new("lfz", bytes);
            let err = load_file(&f.path()).expect_err("应为 CosmosAnswerError");
            assert_eq!(err.class_name(), "CosmosAnswerError", "bytes={bytes:?}");
            assert_eq!(err.message(), "你忘记了宇宙的答案", "bytes={bytes:?}");
            assert_eq!(err.span(), Some(Span::START), "bytes={bytes:?}");
        }
    }

    // ---- 前导消费 / line_base ----

    #[test]
    fn valid_lfz_preamble_sets_line_base_one() {
        let f = TempFile::new("lfz", b"#42\nhello\nworld\n");
        let l = load_file(&f.path()).unwrap();
        assert_eq!(l.text, "hello\nworld\n", "text 不含前导行");
        assert_eq!(l.line_base, 1);

        let f = TempFile::new("lfz", b"#42\n");
        let l = load_file(&f.path()).unwrap();
        assert_eq!(l.text, "", "空程序体合法");
        assert_eq!(l.line_base, 1);
    }

    #[test]
    fn non_lfz_file_skips_preamble_check() {
        let f = TempFile::new("txt", b"#42\nhello");
        let l = load_file(&f.path()).unwrap();
        assert_eq!(l.text, "#42\nhello", "非 .lfz 不消费前导");
        assert_eq!(l.line_base, 0);

        let f = TempFile::new("", b"#42\nhello");
        let l = load_file(&f.path()).unwrap();
        assert_eq!(l.text, "#42\nhello", "无扩展名文件同样豁免");
        assert_eq!(l.line_base, 0);
    }

    #[test]
    fn empty_non_lfz_file_is_legal_empty_program() {
        let f = TempFile::new("txt", b"");
        let l = load_file(&f.path()).expect("非 .lfz 空文件合法");
        assert_eq!(l.text, "");
        assert_eq!(l.line_base, 0);
    }

    #[test]
    fn load_source_does_not_require_or_consume_preamble() {
        let l = load_source("#42\nhello").unwrap();
        assert_eq!(l.text, "#42\nhello", "REPL/stdin/-e 不消费前导");
        assert_eq!(l.line_base, 0);

        let l = load_source("").unwrap();
        assert_eq!(l.text, "");
        assert_eq!(l.line_base, 0);
    }

    // ---- 非 UTF-8 ----

    #[test]
    fn invalid_utf8_is_syntax_error_with_offset() {
        let f = TempFile::new("txt", &[0xFF, 0xFE]);
        let err = load_file(&f.path()).expect_err("非 UTF-8 应报 SyntaxError");
        assert_eq!(err.class_name(), "SyntaxError");
        assert_eq!(
            err.message(),
            "文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 0）"
        );

        let mut bytes = b"abc".to_vec();
        bytes.push(0xFF);
        let f = TempFile::new("txt", &bytes);
        let err = load_file(&f.path()).unwrap_err();
        assert_eq!(err.class_name(), "SyntaxError");
        assert_eq!(
            err.message(),
            "文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 3）"
        );
    }

    #[test]
    fn invalid_utf8_beats_missing_preamble() {
        // 先编码、后前导：合法 `#42\n` 之后跟非法字节 → SyntaxError（非 CosmosAnswer）。
        let mut bytes = b"#42\n".to_vec();
        bytes.push(0xFF);
        let f = TempFile::new("lfz", &bytes);
        let err = load_file(&f.path()).expect_err("编码错误优先于前导");
        assert_eq!(err.class_name(), "SyntaxError");
        assert_eq!(
            err.message(),
            "文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 4）"
        );

        // 首字节即非法（连前导都没有）也报 SyntaxError。
        let f = TempFile::new("lfz", &[0xFF]);
        let err = load_file(&f.path()).unwrap_err();
        assert_eq!(err.class_name(), "SyntaxError");
    }

    // ---- IO 失败 ----

    #[test]
    fn missing_file_is_io_error_without_span() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path = std::env::temp_dir()
            .join(format!("lfz_definitely_missing_{}_{}.lfz", std::process::id(), nanos))
            .to_str()
            .expect("临时路径应为 UTF-8")
            .to_string();
        let err = load_file(&path).expect_err("缺失文件应报 IOError");
        assert_eq!(err.class_name(), "IOError");
        assert_eq!(err.message(), format!("无法读取：{path}"));
        assert_eq!(err.span(), None, "IO 失败无源码位置");
    }
}
