// super:: 現在のコードの1つ上を表すパス
use super::{parser::AST, Instruction};
// crete:: 現在のクレートのトップを表すパス
use crete::helper::safe_add;
use std::{
    error::Error,
    fmt::{self, Display},
};

/// コード生成のエラーを表す型
#[derive(Debug)]
pub enum CodeGenError {
    PCoverFlow, // コード生成中にオーバーフローが起きた場合
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}
