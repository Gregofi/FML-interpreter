use serde::{Serialize, Deserialize};

#[derive(PartialEq,Debug,Serialize,Deserialize,Clone)]
pub enum AST {
    Integer(i32),
    Boolean(bool),
    Null,

    Variable { name: String, value: Box<AST> },
    Array { size: Box<AST>, value: Box<AST> },
    Object { extends: Box<AST>, members: Vec<Box<AST>> },

    AccessVariable { name: String },
    AccessField { object: Box<AST>, field: String },
    AccessArray { array: Box<AST>, index: Box<AST> },

    AssignVariable { name: String, value: Box<AST> },
    AssignField { object: Box<AST>, field: String, value: Box<AST> },
    AssignArray { array: Box<AST>, index: Box<AST>, value: Box<AST> },

    Function { name: String, parameters: Vec<String>, body: Box<AST> },
    // Operator { operator: Operator, parameters: Vec<String>, body: Box<AST> },    // TODO Consider merging with function

    CallFunction { name: String, arguments: Vec<Box<AST>> },
    CallMethod { object: Box<AST>, name: String, arguments: Vec<Box<AST>> },
    // CallOperator { object: Box<AST>, operator: Operator, arguments: Vec<Box<AST>> }, // TODO Consider removing
    //Operation { operator: Operator, left: Box<AST>, right: Box<AST> },               // TODO Consider removing

    Top (Vec<Box<AST>>),
    Block (Vec<Box<AST>>),
    Loop { condition: Box<AST>, body: Box<AST> },
    Conditional { condition: Box<AST>, consequent: Box<AST>, alternative: Box<AST> },

    Print { format: String, arguments: Vec<Box<AST>> },
}

impl AST {
    /** To make it easier to create boxes */
    #[allow(dead_code)]
    pub fn into_boxed(self) -> Box<Self> {
        Box::new(self)
    }

}
