use smol_str::SmolStr;
use std::cmp::PartialEq;

#[derive(PartialEq, Eq,Debug, Clone)]
pub enum ContextType {
    Loop,
    Func,
    Root,
}

#[derive(PartialEq, Eq,Debug, Clone)]
#[allow(dead_code)] //TODO
pub enum ElementType {
    Argument,
    Library(SmolStr), // SmolStr: 导入名
    Function(usize), // usize: 形参个数
    Value,
    Func,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] //TODO
pub struct Element {
    name: SmolStr,
    el_type: ElementType,
}

#[derive(Debug, Clone)]
pub struct Context {
    elements: Vec<Element>,
    ctxt_type: ContextType,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    contexts: Vec<Context>
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            contexts: vec![]
        };
        table.contexts.push(Context {
            elements: vec![],
            ctxt_type: ContextType::Root,
        });
        table
    }

    // 验证名称是否存在
    pub fn check_element(&self, name: &str) -> bool {
        for context in &self.contexts {
            for el in &context.elements {
                if el.name.as_str() == name {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_element_type(&self, name: &str) -> Option<&ElementType> {
        for context in &self.contexts {
            for el in &context.elements {
                if el.name.as_str() == name {
                    return Some(&el.el_type);
                }
            }
        }
        None
    }

    pub fn add_context(&mut self, ctxt_type: ContextType) {
        self.contexts.push(Context {
            elements: vec![],
            ctxt_type,
        });
    }

    pub fn exit_context(&mut self) {
        self.contexts.pop();
    }

    // 添加一个符号到顶层上下文
    pub fn add_element(&mut self, name: SmolStr, el_type: ElementType) {
        let peek_context = self.contexts.last_mut().unwrap();
        peek_context.elements.push(Element { name, el_type });
    }

    pub fn get_context(&mut self, target_type: &ContextType) -> Option<&mut Context> {
        self.contexts.iter_mut()
            .rev()
            .find(|c| c.ctxt_type == *target_type)
    }
}
