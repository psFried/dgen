use generator::GeneratorType;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub function_name: String,
    pub args: Vec<Expr>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Function(FunctionCall),
    StringLiteral(String),
    IntLiteral(u64),
    SignedIntLiteral(i64),
    DecimalLiteral(f64),
    BooleanLiteral(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSpec {
    pub column_name: String,
    pub spec: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroArgument {
    pub name: String,
    pub arg_type: GeneratorType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub name: String,
    pub args: Vec<MacroArgument>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub assignments: Vec<MacroDef>,
    pub expr: Expr,
}

