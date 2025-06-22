use std::ops::Range;

use chumsky::error::Rich;
use derive_more::Constructor;
use getset::{CopyGetters, Getters};
use string_interner::{symbol::SymbolU32, DefaultBackend, StringInterner};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(SymbolU32);

#[derive(Debug, Clone, Constructor)]
pub struct DebugInfo {
    site: Site
}

#[derive(Debug, Clone, Getters, CopyGetters, Constructor)]
pub struct Site {
    #[get_copy = "pub"]
    file: FileId,

    #[get = "pub"]
    span: Range<usize>,
}

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct SiteError {
    site: Site,
    reason: String
}

#[derive(Debug)]
pub enum Error {
    Simple(String),
    Site(Vec<SiteError>)
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Simple(value)
    }
}

impl Error {
    pub fn from_parse_errors<'str>(file_id: FileId, errors: Vec<Rich<'str, char>>) -> Self {
        Self::Site(
            errors
                .iter()
                .map(|e|
                    SiteError {
                        site: Site {
                            file: file_id,
                            span: e.span().into_range(),
                        },
                        reason: format!("{}", e.reason()),
                    }
                )
                .collect()
        )
    }
}

pub struct DebugContext {
    interner: StringInterner<DefaultBackend>,
}

impl DebugContext {
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
        }
    }

    pub fn get_file_id(&mut self, file_id: &str) -> FileId {
        let id = self.interner.get_or_intern(file_id);
        FileId(id)
    }
}
