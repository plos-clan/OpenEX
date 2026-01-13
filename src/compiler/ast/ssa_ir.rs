use dashu::float::FBig;
use dashu::float::round::mode::HalfAway;
use linked_hash_map::LinkedHashMap;
use slotmap::{DefaultKey, SlotMap};
use smol_str::SmolStr;
use std::collections::{BTreeMap, HashMap};

use crate::compiler::lexer::Token;

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LocalAddr {
    pub offset: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Val(DefaultKey),
    Library(SmolStr),
    Null,
    This,
    ImmBool(bool),
    ImmNum(i64),
    ImmFlot(FBig<HalfAway, 10>),
    ImmStr(SmolStr),
    Call(SmolStr),
    Reference(SmolStr), // 对象引用
    Expression(Box<Operand>, Box<Operand>, Box<OpCode>),
    ImmNumFlot, // 类型占位符
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueGuessType {
    Bool,
    Number,
    String,
    Float,
    Null,
    Ref,
    This,
    Unknown,
    Array,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value {
    pub(crate) variable: bool,        // 是否被重赋值
    pub(crate) type_: ValueGuessType, // 猜测类型
    token: Token,                     // 变量名 token
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // TODO
pub enum OpCode {
    // Option<LocalAddr> 为各 IR 的逻辑地址
    LoadGlobal(Option<LocalAddr>, DefaultKey, Operand), // 栈顶元素加载到全局变量
    StoreGlobal(Option<LocalAddr>, DefaultKey, Operand), // 将一个全局变量加载到栈顶
    LoadLocal(Option<LocalAddr>, DefaultKey, Operand),  // 栈顶元素加载到局部变量
    StoreLocal(Option<LocalAddr>, DefaultKey, Operand), // 将一个变量加载到栈顶
    LoadArrayLocal(Option<LocalAddr>, DefaultKey, usize), // 将指定栈顶元素组合成数组加载到局部变量表
    LoadArrayGlobal(Option<LocalAddr>, DefaultKey, usize), // 将指定栈顶元素组合成数组加载到全局变量表
    SetArrayLocal(Option<LocalAddr>, DefaultKey),          // 将栈顶元素设置进数组指定索引
    SetArrayGlobal(Option<LocalAddr>, DefaultKey),         // 将栈顶元素设置进数组指定索引
    Push(Option<LocalAddr>, Operand),                      // 将值压入操作栈
    Pop(Option<LocalAddr>, usize),                         // 弹出操作栈顶部的值
    AddLocalImm(Option<LocalAddr>, DefaultKey, i64),       // 局部变量 += 立即数
    Call(Option<LocalAddr>, SmolStr),                      // 函数调用
    Jump(Option<LocalAddr>, Option<LocalAddr>),            // 无条件跳转
    JumpTrue(Option<LocalAddr>, Option<LocalAddr>, Operand), // 栈顶结果为真则跳转
    JumpFalse(Option<LocalAddr>, Option<LocalAddr>, Operand), // 栈顶结构为假则跳转
    LazyJump(Option<LocalAddr>, Option<LocalAddr>, bool),  // 懒跳转 (是否是 break)
    Return(Option<LocalAddr>),                             // 栈顶结果返回
    Nop(Option<LocalAddr>),                                // 空操作

    Pos(Option<LocalAddr>), // +
    Neg(Option<LocalAddr>), // -

    Add(Option<LocalAddr>), // +
    Sub(Option<LocalAddr>), // -
    Mul(Option<LocalAddr>), // *
    Div(Option<LocalAddr>), // /
    And(Option<LocalAddr>), // &&
    Or(Option<LocalAddr>),  // ||
    Rmd(Option<LocalAddr>), // %

    Equ(Option<LocalAddr>),    // ==
    NotEqu(Option<LocalAddr>), // !=
    BigEqu(Option<LocalAddr>), // >=
    LesEqu(Option<LocalAddr>), // <=
    Big(Option<LocalAddr>),    // >
    Less(Option<LocalAddr>),   // <

    SAdd(Option<LocalAddr>), // ++
    SSub(Option<LocalAddr>), // --

    Not(Option<LocalAddr>), // !

    Store(Option<LocalAddr>), // =
    AddS(Option<LocalAddr>),  // +=
    SubS(Option<LocalAddr>),  // -=
    MulS(Option<LocalAddr>),  // *=
    DivS(Option<LocalAddr>),  // /=
    RmdS(Option<LocalAddr>),  // %=

    BitAnd(Option<LocalAddr>), // &
    BitOr(Option<LocalAddr>),  // |
    BitXor(Option<LocalAddr>), // ^

    BAndS(Option<LocalAddr>), // &=
    BOrS(Option<LocalAddr>),  // |=
    BXorS(Option<LocalAddr>), // ^=

    BLeft(Option<LocalAddr>),  // <<
    BRight(Option<LocalAddr>), // >>

    Ref(Option<LocalAddr>),    // .
    AIndex(Option<LocalAddr>), // 数组索引
}

impl OpCode {
    fn relocate_addr(&mut self, addr_map: &HashMap<LocalAddr, LocalAddr>) {
        // 重定位第一个字段：Option<LocalAddr>
        if let Some(new_addr) = addr_map.get(&self.get_id()) {
            self.set_id(*new_addr);
        }

        // 重定位 Jump 和 JumpTrue 的跳转目标
        match self {
            Self::JumpTrue(_, target, ..)
            | Self::JumpFalse(_, target, ..)
            | Self::Jump(_, target)
            | Self::LazyJump(_, target, ..) => {
                if let Some(j_target) = target
                    && let Some(&new_target) = addr_map.get(j_target)
                {
                    *target = Some(new_target);
                }
            }
            // 其他 OpCode 没有额外的 LocalAddr 字段
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OpCodeTable {
    pub(crate) opcodes: LinkedHashMap<LocalAddr, OpCode>,
    alloc_addr: LocalAddr,
}

impl Default for OpCodeTable {
    fn default() -> Self {
        Self::new()
    }
}

impl OpCodeTable {
    #[must_use]
    pub fn new() -> Self {
        Self {
            opcodes: LinkedHashMap::new(),
            alloc_addr: LocalAddr { offset: 0 },
        }
    }

    pub fn add_opcode(&mut self, opcode: OpCode) -> LocalAddr {
        let addr = self.alloc_addr;
        self.opcodes.insert(addr, opcode);
        self.alloc_addr.offset += 1;
        if let Some(op) = self.opcodes.get_mut(&addr) {
            op.set_id(addr);
        } else {
            unreachable!()
        }
        addr
    }

    // 返回IR块第一条和最后一条IR的逻辑地址
    pub fn append_code(&mut self, code: &Self) -> (LocalAddr, Option<LocalAddr>) {
        let start_offset = self.alloc_addr.offset;
        if code.opcodes.is_empty() {
            return (
                LocalAddr {
                    offset: self.alloc_addr.offset,
                },
                None,
            );
        }
        let mut old_to_new: HashMap<LocalAddr, LocalAddr> = HashMap::new();

        // 遍历所有 opcode（按 offset 顺序）
        let mut entries: Vec<_> = code.opcodes.iter().collect();
        entries.sort_unstable_by_key(|&(addr, _)| addr.offset);

        // 为每个旧地址分配新地址
        for (old_addr, _) in &entries {
            let od = **old_addr;
            old_to_new.insert(
                od,
                LocalAddr {
                    offset: self.alloc_addr.offset,
                },
            );
            self.alloc_addr.offset += 1;
        }

        // 插入并重定位
        let mut last_addr = None;
        for (old_addr, op) in entries {
            let mut new_op = op.clone();
            new_op.relocate_addr(&old_to_new);

            let new_addr = old_to_new[old_addr];
            self.opcodes.insert(new_addr, new_op);
            last_addr = Some(new_addr);
        }

        (
            LocalAddr {
                offset: start_offset,
            },
            last_addr,
        )
    }

    pub fn find_code_mut(&mut self, key: LocalAddr) -> Option<&mut OpCode> {
        self.opcodes.get_mut(&key)
    }

    pub fn change_code(&mut self, f: impl FnOnce(&mut Self)) {
        f(self);
    }
}

// SSA_IR 局部变量映射表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalMap {
    pub(crate) locals: BTreeMap<DefaultKey, usize>, // 局部变量表映射
    pub(crate) now_index: usize,
}

impl Default for LocalMap {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalMap {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            locals: BTreeMap::new(),
            now_index: 0,
        }
    }

    pub fn add_local(&mut self, local: DefaultKey) -> usize {
        self.locals.insert(local, self.now_index);
        let ret_m = self.now_index;
        self.now_index += 1;
        ret_m
    }

    #[must_use]
    pub fn get_index(&self, key: DefaultKey) -> Option<&usize> {
        self.locals.get(&key)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub(crate) name: SmolStr,
    pub(crate) args: usize,
    pub(crate) codes: Option<OpCodeTable>, // 为 None 代表本地方法实现
    pub(crate) locals: LocalMap,           // 局部变量表映射
}

#[derive(Debug, Clone)]
pub struct ValueAlloc {
    values: SlotMap<DefaultKey, Value>,
}

impl Default for ValueAlloc {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueAlloc {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: SlotMap::new(),
        }
    }

    pub fn find_value_key(&mut self, name: &SmolStr) -> Option<DefaultKey> {
        for (key, value) in &mut self.values {
            if value.token.text() == name {
                return Some(key);
            }
        }
        None
    }

    pub fn find_value(&mut self, key: DefaultKey) -> Option<&mut Value> {
        self.values.get_mut(key)
    }

    pub fn alloc_value(&mut self, token: Token, type_: ValueGuessType) -> DefaultKey {
        let va = Value {
            variable: false,
            token,
            type_,
        };
        self.values.insert(va)
    }

    /// 用于提取另一个分配表中的引用
    pub fn append_ref(&mut self, value_alloc: &Self) {
        for val in value_alloc.values.values() {
            if val.type_ == ValueGuessType::Ref {
                self.values.insert(val.clone());
            }
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] //TODO
pub struct Code {
    codes: OpCodeTable,
    pub(crate) funcs: Vec<Function>,
    root: bool, // 是否是根脚本上下文 (true: 根上下文|false: 函数上下文)
}

impl Code {
    #[must_use]
    pub fn new(root: bool) -> Self {
        Self {
            codes: OpCodeTable::new(),
            funcs: Vec::new(),
            root,
        }
    }

    pub const fn get_code_table(&mut self) -> &mut OpCodeTable {
        &mut self.codes
    }

    pub fn add_function(&mut self, func: Function) {
        self.funcs.push(func);
    }

    pub fn find_function(&mut self, key: &SmolStr) -> Option<&mut Function> {
        let mut ret_m = None;
        for i in 0..self.funcs.len() {
            if self.funcs.get_mut(i)?.name == *key {
                ret_m = Some(self.funcs.get_mut(i)?);
                break;
            }
        }
        ret_m
    }
}

macro_rules! match_opcodes {
    ($expr: expr,$slot:ident,$stmt: expr) => {
        match $expr {
            OpCode::LoadGlobal($slot, ..)
            | OpCode::StoreGlobal($slot, ..)
            | OpCode::LoadLocal($slot, ..)
            | OpCode::StoreLocal($slot, ..)
            | OpCode::LoadArrayLocal($slot, ..)
            | OpCode::LoadArrayGlobal($slot, ..)
            | OpCode::SetArrayLocal($slot, ..)
            | OpCode::SetArrayGlobal($slot, ..)
            | OpCode::LazyJump($slot, ..)
            | OpCode::Push($slot, ..)
            | OpCode::Pop($slot, ..)
            | OpCode::AddLocalImm($slot, ..)
            | OpCode::Call($slot, ..)
            | OpCode::Jump($slot, ..)
            | OpCode::JumpTrue($slot, ..)
            | OpCode::JumpFalse($slot, ..)
            | OpCode::Return($slot)
            | OpCode::Pos($slot)
            | OpCode::Neg($slot)
            | OpCode::Add($slot)
            | OpCode::Sub($slot)
            | OpCode::Mul($slot)
            | OpCode::Div($slot)
            | OpCode::And($slot)
            | OpCode::Or($slot)
            | OpCode::Rmd($slot)
            | OpCode::Equ($slot)
            | OpCode::NotEqu($slot)
            | OpCode::BigEqu($slot)
            | OpCode::LesEqu($slot)
            | OpCode::Big($slot)
            | OpCode::Less($slot)
            | OpCode::SAdd($slot)
            | OpCode::SSub($slot)
            | OpCode::Not($slot)
            | OpCode::Store($slot)
            | OpCode::AddS($slot)
            | OpCode::SubS($slot)
            | OpCode::MulS($slot)
            | OpCode::DivS($slot)
            | OpCode::RmdS($slot)
            | OpCode::BitAnd($slot)
            | OpCode::BitOr($slot)
            | OpCode::BitXor($slot)
            | OpCode::BAndS($slot)
            | OpCode::BOrS($slot)
            | OpCode::BXorS($slot)
            | OpCode::BLeft($slot)
            | OpCode::BRight($slot)
            | OpCode::Ref($slot)
            | OpCode::AIndex($slot)
            | OpCode::Nop($slot) => $stmt,
        }
    };
}

impl OpCode {
    #[must_use]
    pub const fn get_id(&self) -> LocalAddr {
        match_opcodes!(self, slot, slot).unwrap()
    }

    pub const fn set_id(&mut self, id: LocalAddr) {
        match_opcodes!(self, slot, *slot = Some(id));
    }
}
