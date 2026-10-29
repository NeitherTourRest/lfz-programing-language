//! T16-A 来源元数据；不加载文件，不持有 AST 或运行时值。
//! 精确契约见 docs/spec/interface-contract.md §19。
use crate::error::LzError;
use crate::loader::Loaded;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub u32);
impl SourceId { pub const ENTRY: Self = Self(0); }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);
impl ModuleId { pub const ENTRY: Self = Self(0); }

#[derive(Clone, Debug)]
pub struct SourceRecord {
    pub identity: Option<PathBuf>,
    pub display_path: String,
    pub loaded: Option<Loaded>,
}

#[derive(Clone, Debug, Default)]
pub struct SourceMap { records: Vec<SourceRecord> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceMetadataError {
    TooManySources,
    FrameCount { expected: usize, actual: usize },
    UnknownSource { id: SourceId },
    UnexpectedErrorSource,
    MissingErrorSource,
}

#[derive(Debug)]
pub struct LocatedFailure {
    pub source_id: SourceId,
    pub error: Box<LzError>,
}

fn checked_source_id(len: usize) -> Result<SourceId, SourceMetadataError> {
    u32::try_from(len).map(SourceId).map_err(|_| SourceMetadataError::TooManySources)
}

impl SourceMap {
    pub fn new() -> Self { Self::default() }
    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
    pub fn get(&self, id: SourceId) -> Option<&SourceRecord> {
        usize::try_from(id.0).ok().and_then(|index| self.records.get(index))
    }
    pub fn insert(&mut self, record: SourceRecord) -> Result<SourceId, SourceMetadataError> {
        // 先完整验证编号域，再修改集合；宿主分配失败不伪装成语言错误。
        let id = checked_source_id(self.records.len())?;
        self.records.push(record);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SyntaxMsg;
    use crate::span::Span;
    fn record(name: &str) -> SourceRecord {
        SourceRecord { identity: None, display_path: name.into(), loaded: None }
    }
    #[test]
    fn ids_are_separate_zero_entry_domains() {
        let s = SourceId::ENTRY; let m = ModuleId::ENTRY;
        assert_eq!(s, SourceId(0)); assert_eq!(m, ModuleId(0));
        let mut ids = std::collections::HashSet::new();
        assert!(ids.insert(s)); assert!(!ids.insert(SourceId(0)));
    }
    #[test]
    fn insertion_is_stable_and_unknown_ids_do_not_fall_back() {
        let mut map = SourceMap::new();
        assert!(map.is_empty()); assert_eq!(map.len(), 0);
        assert!(map.get(SourceId::ENTRY).is_none());
        assert_eq!(map.insert(record("<command>")), Ok(SourceId::ENTRY));
        assert_eq!(map.insert(record("b.lfz")), Ok(SourceId(1)));
        assert!(!map.is_empty()); assert_eq!(map.len(), 2);
        assert_eq!(map.get(SourceId(0)).unwrap().display_path, "<command>");
        assert_eq!(map.get(SourceId(1)).unwrap().display_path, "b.lfz");
        assert!(map.get(SourceId(2)).is_none()); assert!(map.get(SourceId(u32::MAX)).is_none());
        assert_eq!(map.get(SourceId(0)).unwrap().display_path, "<command>");
    }
    #[test]
    fn record_keeps_loaded_bytes_line_base_and_identity_without_io() {
        let raw = "\u{feff}甲\r\n乙\r";
        let mut map = SourceMap::new();
        let identity = PathBuf::from("does-not-exist/../NoSuch.LFZ");
        let id = map.insert(SourceRecord { identity: Some(identity.clone()), display_path: "虚拟/NoSuch.LFZ".into(), loaded: Some(Loaded { text: raw.into(), line_base: 7 }) }).unwrap();
        let r = map.get(id).unwrap();
        assert_eq!(r.identity.as_ref(), Some(&identity));
        assert_eq!(r.display_path, "虚拟/NoSuch.LFZ");
        assert_eq!(r.loaded.as_ref().unwrap().text.as_bytes(), raw.as_bytes());
        assert_eq!(r.loaded.as_ref().unwrap().line_base, 7);
        let cloned = map.clone();
        assert_eq!(cloned.get(id).unwrap().loaded, r.loaded);
    }
    #[test]
    fn failed_loading_can_have_source_without_loaded_text() {
        let mut map = SourceMap::default();
        let id = map.insert(record("missing.lfz")).unwrap();
        assert!(map.get(id).unwrap().loaded.is_none());
        assert!(map.get(id).unwrap().identity.is_none());
    }
    #[test]
    fn checked_id_boundary_without_mass_allocation() {
        assert_eq!(checked_source_id(0), Ok(SourceId::ENTRY));
        assert_eq!(checked_source_id(u32::MAX as usize), Ok(SourceId(u32::MAX)));
        if let Ok(too_many) = usize::try_from(u64::from(u32::MAX) + 1) {
            assert_eq!(checked_source_id(too_many), Err(SourceMetadataError::TooManySources));
        }
    }
    #[test]
    fn located_failure_keeps_existing_error_and_public_shapes() {
        let span = Span { line: 4, col: 9 };
        let old = LzError::Syntax { msg: SyntaxMsg::IncompleteExpr, span };
        let class = old.class_name(); let message = old.message();
        let failure = LocatedFailure { source_id: SourceId(3), error: Box::new(old.clone()) };
        assert_eq!(failure.source_id, SourceId(3)); assert_eq!(*failure.error, old);
        assert_eq!(failure.error.class_name(), class); assert_eq!(failure.error.message(), message);
        assert_eq!(failure.error.span(), Some(span));
        let _frame = crate::error::TraceFrame { func_id: 0, span };
        let _program = crate::ast::Program { span, stmts: vec![] };
    }
}
