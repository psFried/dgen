use generator::{GeneratorArg, ConstantGenerator};

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


#[derive(Debug, PartialEq)]
pub struct ResolveError {
    pub token: Token,
}

impl Token {

    fn into_generator(self) -> Result<GeneratorArg, ResolveError> {
        match self {
            Token::StringLiteral(val) => Ok(GeneratorArg::String(ConstantGenerator::create(val))),
            Token::IntLiteral(int) => Ok(GeneratorArg::SignedInt(ConstantGenerator::create(int))),
            Token::DecimalLiteral(float) => Ok(GeneratorArg::Decimal(ConstantGenerator::create(float))),
            Token::Function(call) => resolve_function_call(call)
        }
    }

}

fn resolve_function_call(function_call: FunctionCall) -> Result<GeneratorArg, ResolveError> {
    unimplemented!()
}
