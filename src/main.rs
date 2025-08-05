use core::panic;
use std::{
    fmt::{Debug, Display},
    iter::{self, Chain, Once, Peekable},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    slice,
    str::{CharIndices, Chars},
    time::Instant,
};

use futures::io::empty;
use serde_json::map::Iter;
#[derive(Clone, Copy)]
struct Solution;

struct Stack<T> {
    root: Option<Box<Node<T>>>,
}

struct Node<T> {
    val: T,
    next: Option<Box<Node<T>>>,
}
impl<T> Node<T> {
    pub fn new(val: T) -> Self {
        Node {
            val: val,
            next: None,
        }
    }
}

impl<T> Stack<T> {
    fn new(val: T) -> Self {
        Stack {
            root: Some(Box::new(Node::new(val))),
        }
    }
    fn empty() -> Self {
        Stack { root: None }
    }

    fn push(&mut self, val: T) {
        let mut new_node = Node::new(val);

        new_node.next = self.root.take();

        self.root = Some(Box::new(new_node));
    }

    fn pop(&mut self) -> Option<T> {
        let mut root = self.root.take()?;
        self.root = root.next.take();

        Some(root.val)
    }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

struct StackIter<'a, T> {
    curr: Option<&'a Node<T>>,
}

struct StackIterMut<'a, T> {
    curr: Option<*mut Node<T>>,
    life_time: PhantomData<&'a ()>,
}

impl<'a, T: 'a> IntoIterator for &'a mut Stack<T> {
    type Item = &'a mut T;
    type IntoIter = StackIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        let mut stack_iter = StackIterMut {
            curr: None,
            life_time: PhantomData,
        };

        stack_iter.curr =
            self.root.as_deref_mut().map(|val| &mut *val as *mut _);

        stack_iter
    }
}

impl<'a, T: 'a> Iterator for StackIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|node| {
            let node: &mut Node<T> = unsafe { &mut *node };
            self.curr = node.next.as_deref_mut().map(|val| &mut *val as *mut _);
            &mut node.val
        })
    }
}

impl<'a, T: 'a> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = StackIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        StackIter {
            curr: self.root.as_deref().map(|val| &*val),
        }
    }
}

impl<'a, T: 'a> Iterator for StackIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|node| {
            self.curr = node.next.as_deref();
            &node.val
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> {
    Let,
    Identifier(&'a str),
    Number(&'a str),
    String(&'a str),
    Func,
    Return,
    Comma,
    Period,
    Mod,
    // Symbols
    Equal,
    Equiv,
    Nequiv,
    Leq,
    Semicolon,
    Colon,
    ParensLeft,
    ParensRight,
    SquareLeft,
    SquareRight,
    CurlyLeft,
    CurlyRight,
    Ampersand,
    Pipe,
    Exclamation,
    LessThan,
    GreaterThan,
    // Flow
    If,
    Else,
    // Loops
    While,
    For,
    // Arithmetic
    Add,
    Sub,
    Mult,
    Div,
    EOF,
    // Booleans
    True,
    False,
    // Types
    Int,
    Bool,
    Unit,
    // Logic
    And,
}
#[derive(Debug)]
pub struct Lexer<'a> {
    source_code: &'a str,
    tokens: [Option<Token<'a>>; 2],
    chars: Peekable<Chain<CharIndices<'a>, Once<(usize, char)>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Lexer<'a> {
        let mut lexer = Lexer {
            chars: source_code
                .char_indices()
                .chain(iter::once((source_code.len(), '\0')))
                .peekable(),

            tokens: [None; 2],
            source_code: &source_code[..],
        };

        lexer.tokens[0] = lexer.next_token();
        lexer.tokens[1] = lexer.next_token();
        lexer
    }
}

impl<'a> Lexer<'a> {
    pub fn current_token(&mut self) -> Option<Token<'a>> {
        self.tokens[0]
    }

    pub fn peek_next_token(&self) -> Option<Token<'a>> {
        self.tokens[1]
    }
}

impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Option<Token<'a>> {
        while let Some((idx, ch)) = self.chars.next() {
            if idx == self.source_code.len() {
                return Some(Token::EOF);
            }

            let tok = match ch {
                ' ' | '\n' => continue,
                ';' => Some(Token::Semicolon),
                ',' => Some(Token::Comma),
                '.' => Some(Token::Period),
                '+' => Some(Token::Add),
                '-' => Some(Token::Sub),
                '*' => Some(Token::Mult),
                '/' => Some(Token::Div),
                '(' => Some(Token::ParensLeft),
                ')' => Some(Token::ParensRight),
                '[' => Some(Token::SquareLeft),
                ']' => Some(Token::SquareRight),
                '{' => Some(Token::CurlyLeft),
                '}' => Some(Token::CurlyRight),
                '|' => Some(Token::Pipe),
                '>' => Some(Token::GreaterThan),
                '%' => Some(Token::Mod),
                ':' => Some(Token::Colon),
                '<' => {
                    if self.chars.next_if(|(_, c)| *c == '=').is_some() {
                        Some(Token::Leq)
                    } else {
                        Some(Token::LessThan)
                    }
                }
                '&' => {
                    if self.chars.next_if(|(_, c)| *c == '&').is_some() {
                        Some(Token::And)
                    } else {
                        Some(Token::Ampersand)
                    }
                }
                '=' => {
                    if self.chars.next_if(|(_, c)| *c == '=').is_some() {
                        Some(Token::Equiv)
                    } else {
                        Some(Token::Equal)
                    }
                }
                '!' => {
                    if self.chars.next_if(|(_, c)| *c == '=').is_some() {
                        Some(Token::Nequiv)
                    } else {
                        Some(Token::Exclamation)
                    }
                }
                '"' => match self.chars.position(|(_, ch)| ch == '"') {
                    Some(pos) => {
                        let tok = Some(Token::String(
                            &self.source_code[idx + 1..idx + pos],
                        ));
                        tok
                    }
                    None => {
                        panic!("expected \", did not find ");
                    }
                },

                x if x.is_alphabetic() => {
                    let mut pos = idx + x.len_utf8();

                    while let Some((_, ch)) = self
                        .chars
                        .next_if(|(_, c)| c.is_alphanumeric() || *c == '_')
                    {
                        pos += ch.len_utf8();
                    }
                    let str = &self.source_code[idx..pos];

                    match str {
                        "let" => Some(Token::Let),
                        "if" => Some(Token::If),
                        "else" => Some(Token::Else),
                        "for" => Some(Token::For),
                        "while" => Some(Token::While),
                        "true" => Some(Token::True),
                        "false" => Some(Token::False),
                        "func" => Some(Token::Func),
                        "return" => Some(Token::Return),
                        "Unit" => Some(Token::Unit),
                        "Int" => Some(Token::Int),
                        "Bool" => Some(Token::Bool),
                        str => Some(Token::Identifier(str)),
                    }
                }
                x if x.is_numeric() => {
                    let mut pos = idx + x.len_utf8();
                    while let Some((_, ch)) =
                        self.chars.next_if(|(_, c)| c.is_numeric())
                    {
                        pos += ch.len_utf8();
                    }

                    let tok = Some(Token::Number(&self.source_code[idx..pos]));
                    tok
                }
                _ => panic!("Unknown character"),
            };
            return tok;
        }
        None
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.next_token();
        let token = self.tokens[0];
        self.tokens[0] = self.tokens[1];
        self.tokens[1] = tok;

        token
    }
}

// impl<'a> Iterator for Lexer<'a>
// where
//     for<'b> Lexer<'a>: 'b,
// {
//     type Item = Token<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.tokens[0]
//     }
// }

// impl Solution {
//     pub fn is_valid(s: String) -> bool {
//         let mut stack = Stack::empty();
//         let matching_char = |(_,c)| match c {
//             ')' => '(',
//             ']' => '[',
//             '}' => '{',
//             _ => panic!("not possible"),
//         };
//         for ch in s.chars() {
//             match ch {
//                 '(' | '[' | '{' => {
//                     stack.push(ch);
//                 }
//                 ch => {
//                     if let Some(c) = stack.pop()
//                         && c == matching_char(ch)
//                     {
//                         continue;
//                     } else {
//                         return false;
//                     }
//                 }
//             }
//         }

//         stack.is_empty()
//     }
// }

impl Solution {
    pub fn is_valid(s: String) -> bool {
        let iter = &mut s.chars().peekable();

        Self::is_valid_req(iter, None) && iter.next().is_none()
    }

    fn matching_char(ch: char) -> char {
        match ch {
            ')' => '(',
            ']' => '[',
            '}' => '{',
            _ => panic!("not possible"),
        }
    }

    pub fn is_valid_req(
        chars: &'_ mut Peekable<Chars<'_>>,
        prev_open: Option<char>,
    ) -> bool {
        let valid = loop {
            match chars.next() {
                Some(open @ ('(' | '[' | '{')) => {
                    println!("opening: {open}");

                    let valid = Self::is_valid_req(chars, Some(open));

                    match (valid, chars.peek(), prev_open) {
                        (true, Some(_), _) => {
                            continue;
                        }

                        (true, None, Some(_)) => {
                            break false;
                        }
                        (true, None, None) => {
                            break true;
                        }

                        _ => break false,
                    }
                }
                Some(close) => {
                    break prev_open.is_some_and(|prev_open| {
                        Self::matching_char(close) == prev_open
                    });
                }

                _ => break false,
            }
        };
        valid
    }
}

pub enum Kind {
    Number,
    String,
    Ident,
}

struct TokenDelux<'a> {
    src: &'a str,
    kind: Kind,
}

fn rand(upper: usize) -> usize {
    let val = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    (val << 2) as usize % upper
}

fn weighted_choice<const N: usize>(
    elements: &[usize; N],
    weights: &[usize; N],
) -> usize {
    let cum_weights: Vec<usize> = weights
        .iter()
        .scan(0_usize, |state, weight| {
            *state += weight;
            Some(*state)
        })
        .collect();
    let rand_number = rand(*cum_weights.last().unwrap());
    for (&element, &weight) in elements.into_iter().zip(&cum_weights) {
        if rand_number < weight {
            return element;
        }
    }
    panic!("haha")
}

// 1) The trait
pub trait Stuffable<T> {
    fn stuff(&self);
}
pub trait Displayable {
    fn stuff(&self);
}

impl<T> Displayable for T
where
    T: Display,
{
    fn stuff(&self) {
        println!("{}", self);
    }
}
// 3a) If you have a Vec of Display‐able items, just print them:
impl<T: Displayable> Stuffable<()> for Vec<T> {
    fn stuff(&self) {
        for val in self {
            val.stuff();
        }
    }
}

impl<T: Stuffable<()>> Stuffable<((), ())> for Vec<T> {
    fn stuff(&self) {
        for val in self {
            val.stuff();
        }
    }
}
impl<T: Stuffable<((),())>> Stuffable<((),(), ())> for Vec<T> {
    fn stuff(&self) {
        for val in self {
            val.stuff();
        }
    }
}




// 3b) If you have a Vec of things that themselves implement Stuffable,

fn main() {
    // Build a 3D “vector of vectors of vectors of i32”:
    let vectors = vec![vec![vec![1, 2, 3]]];

    // The call below:
    // • Innermost: Vec<i32> → prints “1\n2\n3\n” by the first impl
    // • Next level up: Vec<VectorS<i32>> → calls .stuff() on each
    // • Top level: Vec<VectorS<VectorS<i32>>> → same again
    vectors.stuff();
}
