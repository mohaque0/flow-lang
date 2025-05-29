use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringId(usize);

pub struct CommonCtx {
    interned_strings: HashMap<String, StringId>
}