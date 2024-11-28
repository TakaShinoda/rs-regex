//! 正規表現をパースし、抽象構文木に変換
use std::{
    error::Error,
    fmt::{self, Display},
    mem::take, // take はある変数からの所有権の取得と、その変数の初期化を同時に行う
};

/// 抽象構文木を表現するための型
/// ```
/// AST::Seq(vec![AST::Char('a'), AST::Char('b'), AST::Char('c')])
/// ```
#[derive(Debug)]
pub enum AST {
    Char(char),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>), // 正規表現の列を表現する
}

/// パースエラーを表すための型
#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char), // 誤ったエスケープシーケンス
    invalidRightParen(usize),   // 開き括弧なし
    NoPrev(usize),              // +, |, *, ? の前に式がない
    NoRightParen,               // 閉じ括弧なし
    Empty,                      // 空のパターン
}

/// パースエラーを表示するために、Display トレイトを実装
impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid espace: pos = {pos}, char = '{c}'")
            }
            ParseError::invalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}'")
            }
            ParseError::NoRightParen => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty(pos, c) => write!(f, "ParseError: empty expression"),
        }
    }
}

impl Error for ParseError {}

/// 特殊文字のエスケープ
/// pos: 現在の文字の位置
/// c: エスケープする特殊文字
fn parse_escape(pos: usize, c: char) -> Result<AST, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}
