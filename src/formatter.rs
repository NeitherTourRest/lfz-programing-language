//! Source-preserving whitespace formatting. See interface contract §27.
use crate::{error::LzError, lexer::{self, Token, TokenKind}, loader::{self, Loaded}, parser};
use std::ops::Range;

#[derive(Debug)]
pub enum FormatError {
    InvalidSource { error: Box<LzError>, loaded: Option<Loaded> },
    InvariantViolation,
}

/// Formats the bytes of one valid source file according to its loader path.
/// Use `evaluator::on_eval_stack` for deep input, as for the parser itself.
pub fn format_source(path: &str, raw: &[u8]) -> Result<Vec<u8>, FormatError> {
    let (loaded, tokens) = checked_tokens(path, raw)?;
    let (layout_tokens, layout) = lexer::lex_format_layout(&loaded.text, loaded.line_base)
        .map_err(|_| FormatError::InvariantViolation)?;
    if layout_tokens != tokens {
        return Err(FormatError::InvariantViolation);
    }
    let offsets = raw_scalar_offsets(raw, &loaded).ok_or(FormatError::InvariantViolation)?;
    let output = apply_layout(raw, &offsets, &layout).ok_or(FormatError::InvariantViolation)?;
    let (_, output_tokens) = checked_tokens(path, &output).map_err(|_| FormatError::InvariantViolation)?;
    if tokens.len() != output_tokens.len() || !tokens.iter().zip(&output_tokens).all(|(original, formatted)| {
        original.kind == formatted.kind
            && original.span.line == formatted.span.line
            && (matches!(original.kind, TokenKind::Newline | TokenKind::Eof)
                || original.span.col == formatted.span.col)
    }) {
        return Err(FormatError::InvariantViolation);
    }
    Ok(output)
}

fn checked_tokens(path: &str, raw: &[u8]) -> Result<(Loaded, Vec<Token>), FormatError> {
    let loaded = loader::load_bytes(path, raw.to_vec())
        .map_err(|error| FormatError::InvalidSource { error, loaded: None })?;
    let tokens = match lexer::lex(&loaded.text, loaded.line_base) {
        Ok(tokens) => tokens,
        Err(error) => return Err(FormatError::InvalidSource { error, loaded: Some(loaded) }),
    };
    if let Err(error) = parser::parse(&tokens) {
        return Err(FormatError::InvalidSource { error, loaded: Some(loaded) });
    }
    Ok((loaded, tokens))
}

/// A normalized scalar boundary maps to the start of its raw character, or
/// the first byte of the original line terminator for a normalized newline.
fn raw_scalar_offsets(raw: &[u8], loaded: &Loaded) -> Option<Vec<usize>> {
    let text = std::str::from_utf8(raw).ok()?;
    let mut body_start = if text.starts_with('\u{feff}') { 3 } else { 0 };
    if loaded.line_base == 1 {
        let terminator = raw[body_start..].iter().position(|b| matches!(b, b'\r' | b'\n'))? + body_start;
        body_start = terminator + if raw[terminator..].starts_with(b"\r\n") { 2 } else { 1 };
    }
    let mut original = text[body_start..].char_indices().peekable();
    let mut offsets = Vec::new();
    for normalized in loaded.text.chars() {
        let (offset, ch) = original.next()?;
        let mapped = if ch == '\r' {
            if original.peek().is_some_and(|&(_, next)| next == '\n') {
                original.next();
            }
            '\n'
        } else {
            ch
        };
        if mapped != normalized {
            return None;
        }
        offsets.push(body_start + offset);
    }
    if original.next().is_some() {
        return None;
    }
    offsets.push(raw.len());
    Some(offsets)
}

fn apply_layout(raw: &[u8], offsets: &[usize], layout: &[Range<usize>]) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(raw.len());
    let mut copied = 0;
    for range in layout {
        let start = *offsets.get(range.start)?;
        let end = *offsets.get(range.end)?;
        if start < copied || start >= end || !raw[start..end].iter().all(|b| matches!(b, b' ' | b'\t')) {
            return None;
        }
        let previous = start.checked_sub(1).and_then(|index| raw.get(index));
        let line_start = Some(&start) == offsets.first() || matches!(previous, Some(b'\r' | b'\n'));
        let line_end = matches!(raw.get(end), None | Some(b'\r' | b'\n'));
        let protects_bare_cr = previous == Some(&b'\r') && raw.get(end) == Some(&b'\n');

        output.extend_from_slice(&raw[copied..start]);
        if protects_bare_cr || (!line_end && !line_start) {
            output.extend_from_slice(&raw[start..end]);
        } else if !line_end {
            output.resize(output.len() + end - start, b' ');
        }
        copied = end;
    }
    output.extend_from_slice(&raw[copied..]);
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_raw_protected_regions_and_mixed_line_endings() {
        let raw = "\u{feff}#42\r\n\tlet x = 0xff \t\r\t \n\tlet y = \"中文 ${\t\"内 ${x:>4}\" \t} \t\" \t\r\n\t/* hi \t\r\tinside \t\nend */ \t\r\n\t// keep \t\n\t x |> str() \t\n\t;; \t";
        let expected = "\u{feff}#42\r\n let x = 0xff\r\t \n let y = \"中文 ${\t\"内 ${x:>4}\" \t} \t\"\r\n /* hi \t\r\tinside \t\nend */\r\n // keep \t\n  x |> str()\n ;;";
        let formatted = format_source("x.lfz", raw.as_bytes()).unwrap();
        assert_eq!(formatted, expected.as_bytes());
        assert_eq!(format_source("x.lfz", &formatted).unwrap(), formatted);
    }

    #[test]
    fn empty_comment_and_whitespace_files_preserve_their_terminators() {
        for (path, raw, expected) in [
            ("x.txt", "", ""),
            ("x.lfz", "\u{feff}#42\r", "\u{feff}#42\r"),
            ("x.lfz", "#42\r\t \n", "#42\r\t \n"),
            ("x.txt", "\t \r\n \t\n\t ", "\r\n\n"),
            ("x.txt", "\t// comment \t", " // comment \t"),
            ("x.txt", "\t/* a */ \t1 \t", " /* a */ \t1"),
        ] {
            let formatted = format_source(path, raw.as_bytes()).unwrap();
            assert_eq!(formatted, expected.as_bytes(), "{raw:?}");
            assert_eq!(format_source(path, &formatted).unwrap(), formatted);
        }
    }

    #[test]
    fn invalid_input_uses_original_loader_and_frontend_diagnostics() {
        assert!(matches!(format_source("x.lfz", b"\xff"), Err(FormatError::InvalidSource{loaded:None,..})));
        assert!(matches!(format_source("x.lfz", b"#42 \n1"), Err(FormatError::InvalidSource{loaded:None,..})));
        for raw in [b"#42\n@".as_slice(), b"#42\nlet =".as_slice()] {
            let Err(FormatError::InvalidSource { error, loaded: Some(source) }) = format_source("x.lfz", raw) else {
                panic!("expected original source diagnostic");
            };
            assert_eq!(source.line_base, 1);
            assert_eq!(error.span().unwrap().line, 2);
        }
    }
}
