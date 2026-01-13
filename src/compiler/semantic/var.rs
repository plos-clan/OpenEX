use crate::compiler::ast::ASTExprTree;
use crate::compiler::ast::ssa_ir::OpCode::{
    Call, LoadArrayGlobal, LoadArrayLocal, LoadGlobal, LoadLocal, Push,
};
use crate::compiler::ast::ssa_ir::{LocalMap, OpCodeTable, Operand, ValueAlloc, ValueGuessType};
use crate::compiler::lexer::Token;
use crate::compiler::parser::ParserError;
use crate::compiler::parser::symbol_table::ElementType::Value;
use crate::compiler::semantic::Semantic;
use crate::compiler::semantic::expression::expr_semantic;
use smol_str::SmolStr;

pub fn array_semantic(
    semantic: &mut Semantic,
    name: Token,
    elements: Vec<ASTExprTree>,
    code: &mut ValueAlloc,
    locals: &mut LocalMap,
    root: bool,
) -> Result<OpCodeTable, ParserError> {
    let symbol_table = &mut semantic.compiler_data().symbol_table;
    if symbol_table.check_element(name.text()) {
        return Err(ParserError::SymbolDefined(name));
    }
    symbol_table.add_element(name.value().unwrap(), Value);
    let key = code.alloc_value(name, ValueGuessType::Array);
    let mut opcode_vec = OpCodeTable::new();
    locals.add_local(key);

    let array_length = elements.len();

    for element in elements {
        let ret_m = expr_semantic(semantic, Some(element), code)?;
        opcode_vec.append_code(&ret_m.2);
    }

    if root {
        opcode_vec.add_opcode(LoadArrayGlobal(None, key, array_length));
    } else {
        opcode_vec.add_opcode(LoadArrayLocal(None, key, array_length));
    }

    Ok(opcode_vec)
}

pub fn array_fill_semantic(
    semantic: &mut Semantic,
    name: Token,
    value: ASTExprTree,
    count: ASTExprTree,
    code: &mut ValueAlloc,
    locals: &mut LocalMap,
    root: bool,
) -> Result<OpCodeTable, ParserError> {
    let symbol_table = &mut semantic.compiler_data().symbol_table;
    if symbol_table.check_element(name.text()) {
        return Err(ParserError::SymbolDefined(name));
    }
    symbol_table.add_element(name.value().unwrap(), Value);
    let key = code.alloc_value(name, ValueGuessType::Array);
    locals.add_local(key);
    let mut opcode_vec = OpCodeTable::new();

    let value_expr = expr_semantic(semantic, Some(value), code)?;
    opcode_vec.append_code(&value_expr.2);
    let count_expr = expr_semantic(semantic, Some(count), code)?;
    opcode_vec.append_code(&count_expr.2);
    opcode_vec.add_opcode(Push(
        None,
        Operand::Reference(SmolStr::new("type/array_fill")),
    ));
    opcode_vec.add_opcode(Call(None, SmolStr::new("array_fill")));

    let opread = Operand::Call(SmolStr::new("array_fill"));
    if root {
        opcode_vec.add_opcode(LoadGlobal(None, key, opread));
    } else {
        opcode_vec.add_opcode(LoadLocal(None, key, opread));
    }
    Ok(opcode_vec)
}

pub fn var_semantic(
    semantic: &mut Semantic,
    name: Token,
    init_var: Option<ASTExprTree>,
    code: &mut ValueAlloc,
    root: bool,
    locals: &mut LocalMap,
) -> Result<OpCodeTable, ParserError> {
    let symbol_table = &mut semantic.compiler_data().symbol_table;
    if symbol_table.check_element(name.text()) {
        return Err(ParserError::SymbolDefined(name));
    }
    symbol_table.add_element(name.value().unwrap(), Value);
    let mut opcode_vec = OpCodeTable::new();
    let ret_m = expr_semantic(semantic, init_var, code)?;
    opcode_vec.append_code(&ret_m.2);
    let opread = ret_m.clone();
    let key = code.alloc_value(name, ret_m.1);
    locals.add_local(key);
    if root {
        opcode_vec.add_opcode(LoadGlobal(None, key, opread.0));
    } else {
        opcode_vec.add_opcode(LoadLocal(None, key, opread.0));
    }
    Ok(opcode_vec)
}
