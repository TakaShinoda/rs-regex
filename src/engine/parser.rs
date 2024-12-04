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
    Seq(Vec<AST>), // 正規表現の列を表現する (sequence)
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

/// parse_plus_star_question 関数で利用するための列挙型
enum PSQ {
    Plus,
    Star,
    Question,
}

/// +, *, ? を AST に変換
///
/// 後置記法で、+, *, ? の前にパターンがない場合はエラー
///
/// 例 : *ab, abc|+ などはエラー
fn parse_plus_star_question(
    seq: &mut Vec<AST>, // (abc)+ の時、abc が入る
    ast_type: PSQ,      // 限量子の種類
    pos: usize,         // 限量子の出現する位置
) -> Result<(), ParseError> {
    // pop: seq の最後尾から要素を削除し返す
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            PSQ::Plus => AST::Plus(Box::new(prev)),
            PSQ::Star => AST::Star(Box::new(prev)),
            PSQ::Question => AST::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(());
    } else {
        // 限量子前に限量するパターンが現れないような用い方の時
        Err(ParseError::NoPrev(pos))
    }
}

/// Or で結合された複数の式を AST に変換
///
/// 例: abc|def|ghi は、 AST::Or("abc", AST::Or("def" , "ghi")) という AST となる
fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse(); // AST::Or 先頭の式をASTのルートするため、並びを反転させる
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        seq_or.pop() // seq_or 中の唯一の要素を返す
    }
}

/// 正規表現を正規表現を抽象構文木に変換
/// 引数として受け取った正規表現文字列から1文字ずつ文字を取り出し、それに該当する AST を生成する
pub fn parse(expr: &str) -> Result<AST, ParseError> {
    // 内部状態を表現するための型
    // 関数内で型を定義することで、この関数内でのみ用いる
    // Char: 文字列処理中
    // Escape: エスケープシーケンス処理中
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new(); // 現在の Seq のコンテキスト
    let mut seq_or = Vec::new(); // 現在の Or のコンテキスト
    let mut stack = Vec::new(); // コンテキストのスタック、コンテキストの保存と復元を行う
    let mut state = ParseState::Char; // 現在の状態

    // chars で各文字のイテレータを取得
    // enumerate で繰り返し番号とイテレータのペアが返る
    // 番号はエラー時に、エラーが起きた場所を把握するために使う
    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => {
                match c {
                    '+' => parse_plus_star_question(&mut seq, PSQ::Plus, i)?,
                    '*' => parse_plus_star_question(&mut seq, PSQ::Star, i)?,
                    '?' => parse_plus_star_question(&mut seq, PSQ::Question, i)?,
                    '(' => {
                        // 現在のコンテキストをスタックに保存し、
                        // 現在のコンテキストを空の状態にする
                        let prev = take(&mut seq);
                        let prev_or = take(&mut seq_or);
                        stack.push(prev, prev_or);
                    }
                    ')' => {
                        // 現在のコンテキストをスタックからポップ
                        if let Some((mut prev, prev_or)) = stack.pop() {
                            // "()" のように式が空の場合は push しない
                            if !seq.is_empty() {
                                seq_or.push(AST::Seq(seq))
                            }

                            // Or を生成
                            if let Some(ast) = fold_or(seq_or) {
                                prev.push(ast);
                            }

                            // 以前のコンテキストを、現在のコンテキストにする
                            seq = prev;
                            seq_or = prev_or;
                        } else {
                            // "abc)" のように、開き括弧がないのに閉じ括弧がある場合はエラー
                            return Err(Box::new(ParseError::invalidRightParen(i)));
                        }
                    }
                    '|' => {
                        if seq.is_empty() {
                            // "||", "(|abc)" などと、式が空の場合はエラー
                            return Err(Box::new(ParseError::NoPrev(i)));
                        } else {
                            let prev = take(&mut seq);
                            seq_or.push(AST::Char(c));
                        }
                    }
                    '\\' => state = ParseState::Escape,
                    _ => seq.push(AST::Char(c)),
                }
            }
            ParseState::Escape => {
                // エスケープシーケンス
                let ast = parse_escape(i, c)?;
                seq.push(ast);
                state = ParseState::Char;
            }
        }
    }

    // 閉じ括弧が足りない場合はエラー
    if !stack.is_empty() {
        return Err(Box::new(ParseError::NoRightParen));
    }

    // "()" のように、式が空の場合は push しない
    if !seq.is_empty() {
        seq_or.push(AST::Seq(seq));
    }

    // Or を生成し、成功した場合はそれを返す
    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(Box::new(ParseError::Empty))
    }
}
