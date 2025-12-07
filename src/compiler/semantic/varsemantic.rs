use crate::compiler::ast::ssa_ir::OpCode::StackLocal;
use crate::compiler::ast::ssa_ir::{Code, OpCode};
use crate::compiler::ast::ASTExprTree;
use crate::compiler::lexer::Token;
use crate::compiler::parser::symbol_table::ElementType::VALUE;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::exprsemantic::expr_semantic;
use crate::compiler::semantic::Semantic;

pub fn var_semantic(
    semantic: &mut Semantic,
    mut name: Token,
    init_var: Option<ASTExprTree>,
    code: &mut Code,
) -> Result<OpCode, ParserError> {
    let symbol_table = &mut semantic.compiler_data().symbol_table;
    if symbol_table.check_element(name.value().unwrap()) {
        return Err(ParserError::SymbolDefined(name));
    }
    symbol_table.add_element(name.value().unwrap(), VALUE);
    let ret_m = expr_semantic(semantic, init_var,code)?;
    let opread = ret_m.clone();
    let key = code.alloc_value(name, ret_m.1);
    Ok(StackLocal(key, opread.0))
}
