use lfz::source::{SourceMap,SourceId};
fn main(){let map=SourceMap::new();assert!(map.get(SourceId::ENTRY).is_none());}
