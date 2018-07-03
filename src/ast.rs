
#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub function_name: String,
    pub args: Vec<Token>
}


#[derive(Debug, PartialEq)]
pub enum Token {
    Function(FunctionCall),
    StringLiteral(String),
    IntLiteral(i64),
    DecimalLiteral(f64),
}

#[derive(Debug, PartialEq)]
pub struct ColumnSpec {
    pub column_name: String,
    pub spec: Token,
}




