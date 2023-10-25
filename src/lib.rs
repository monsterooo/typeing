use config::TypeingConfig;

pub mod config;
pub mod textgen;
pub mod tui;
pub mod wordlists;

use textgen::{RawWordSelector, WordSelector};
use tui::{Text, TypeingTui};

/// 输入测试终端UI和逻辑
pub struct Typeing {
    tui: TypeingTui,
    text: Vec<Text>,
    words: Vec<String>,
    word_selector: Box<dyn WordSelector>,
    config: TypeingConfig,
}

/// 在Typeing中的错误
pub struct TypeingError {
    pub msg: String,
}

/// 转换 [`std::io::Error`] 到 [`TypeingError`]
///
/// 只保留错误信息
impl From<std::io::Error> for TypeingError {
    fn from(value: std::io::Error) -> Self {
        TypeingError {
            msg: value.to_string(),
        }
    }
}

impl From<String> for TypeingError {
    fn from(value: String) -> Self {
        TypeingError { msg: value }
    }
}

impl std::fmt::Debug for TypeingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("TypeingError: {}", self.msg).as_str())
    }
}

impl<'a> Typeing {
    pub fn new(config: TypeingConfig) -> Result<Self, TypeingError> {
        let word_selector: Box<dyn WordSelector> =
            if let Some(wordlist_path) = config.wordlist_file.clone() {

            }
    }
}
