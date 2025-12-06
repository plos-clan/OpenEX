use crate::compiler::lexer::Token;
use std::cmp::PartialEq;

#[derive(PartialEq)]
pub enum ContextType {
    LOOP,
    FUNC,
    IF,
    NONE,
    ROOT,
}

#[derive(PartialEq)]
pub enum ElementType {
    ARGUMENT,
    VALUE,
    FUNC,
}

pub struct Element {
    token: Token,
    el_type: ElementType,
}

pub struct Context {
    elements: Vec<Element>,
    ctxt_type: ContextType,
}

pub struct SymbolTable {
    contexts: Vec<Context>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut table = SymbolTable {
            contexts: vec![]
        };
        table.contexts.push(Context {
            elements: vec![],
            ctxt_type: ContextType::ROOT,
        });
        table
    }

    pub fn add_context(&mut self, ctxt_type: ContextType) {
        self.contexts.push(Context {
            elements: vec![],
            ctxt_type
        });
    }

    pub fn exit_context(&mut self) {
        self.contexts.pop();
    }

    // 添加一个符号到顶层上下文
    pub fn add_element(&mut self, token: Token, el_type: ElementType) {
        let peek_context = self.contexts.last_mut().unwrap();
        peek_context.elements.push(Element { token, el_type });
    }

    pub fn in_context(&self, ctxt_type: ContextType) -> bool {
        for context in &self.contexts {
            if context.ctxt_type == ctxt_type {
                return true;
            }
        }
        false
    }
}
