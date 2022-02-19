use crate::ast::AST;
use std::{collections::HashMap, collections::LinkedList, hash::Hash};

#[derive(Copy, Clone)]
enum Value {
    Int(i32),
    Boolean(bool),
    Unit,
}

struct Program {
    /** Represents environments with variables */
    envs: LinkedList<HashMap<String, Value> >,
}

impl Program {
    pub fn new() -> Self {
        return Program{
            envs: LinkedList::new()
        }
    }

    /// Pushes new environment on top.
    fn push_env(&mut self) {
        self.envs.push_front(HashMap::new());
    }

    /// Pops the top-most environment.
    fn pop_env(&mut self) {
        self.envs.pop_front();
    }

    /// Returns mutable reference to var with 'name' from environments if it exists, 
    /// otherwise returns None.
    /// Scouts the environments from the most recent one.
    fn fetch_var_mut(&mut self, name: &String) -> Option<&mut Value> {
        for env in self.envs.iter_mut() {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var= env.get_mut(name);
            if var.is_some() {
                return var;
            }
        }
        None
    }

    fn assign_to_var(&mut self, name: &String, val: Value) {
        *self.fetch_var_mut(name).expect("Assignment to non-existing variable") = val;
    }

    /// Returns reference to var with 'name' from environments if it exists, 
    /// otherwise returns None.
    /// Scouts the environments from the most recent one.
    fn fetch_var(&self, name: &String) -> Option<&Value> {
        for env in self.envs.iter() {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var= env.get(name);
            if var.is_some() {
                return var;
            }
        }
        None
    }

    /// Adds variable to the top-most environment.
    /// 
    fn add_var(&mut self, name: String, val: Value) {
        let top = self.envs.front_mut().expect("Missing top frame of environment.");
        /* TODO: This probably can be done with expect.
        I was however unable to do it at the time of writing this. */
        match top.try_insert(name, val) {
            Ok(_) => (),
            /* TODO: Make the error message print the variable name. */
            Err(_) => panic!("Variable was redeclared."),
        }
    }

    /// Evaluates AST node as boolean, if the AST node is not boolean then return Err. 
    /// 
    fn eval_bool(&mut self, expr: AST) -> Result<bool, &'static str> {
        match self.eval(expr) {
            Value::Boolean(val) => Ok(val),
            _ => Err("Expression in 'if' condition has to have type bool"),
        }
    }

    /// Evaluates print expression.
    fn eval_print(&mut self, format: String, arguments: Vec<Box<AST>>) {
        let mut vec_it = arguments.into_iter();

        for c in format.chars() {
            match c {
                '~' => {
                    let val = *vec_it.next().expect("Expected more arguments for formatting string.");
                    print!("{}", match self.eval(val) {
                        Value::Int(val) => val.to_string(),
                        Value::Boolean(val) => val.to_string(),
                        Value::Unit => String::from("null"),
                        });
                    },
                _ => print!("{}", c),
            }
        };

    }

    pub fn eval(&mut self, ast: AST) -> Value {
        match ast {
            AST::Integer(val) => Value::Int(val),
          
            AST::Boolean(val) => Value::Boolean(val),
            
            AST::Null => Value::Unit,
         
            AST::Variable { name, value } => {
                let evaluated_val = self.eval(*value);
                self.add_var(name, evaluated_val);
                evaluated_val
            },
           
            AST::Array { size, value } => todo!(),
            AST::Object { extends, members } => todo!(),
            AST::AccessVariable { name } => todo!(),
            AST::AccessField { object, field } => todo!(),
            AST::AccessArray { array, index } => todo!(),
         
            AST::AssignVariable { name, value } => {
                let evaluated = self.eval(*value);
                self.assign_to_var(&name, evaluated);
                evaluated
            },

            AST::AssignField { object, field, value } => todo!(),
            AST::AssignArray { array, index, value } => todo!(),
            AST::Function { name, parameters, body } => todo!(),
            AST::CallFunction { name, arguments } => todo!(),
            AST::CallMethod { object, name, arguments } => todo!(),
            AST::Top(_) => todo!(),

            AST::Block(exprs) => {
                let mut last_val: Option<Value> = None;
                for expr in exprs {
                    self.push_env();
                    last_val = Some(self.eval(*expr));
                }
                self.pop_env();
                match last_val {
                    Some(val) => val,
                    None => Value::Unit,
                }
            },

            AST::Loop { condition, body } => {
                self.push_env();
                loop {
                    if self.eval_bool(*condition.clone()).unwrap() {
                        break;
                    }
                    self.eval(*body.clone());
                }
                self.pop_env();
                Value::Unit
            },

            AST::Conditional { condition, consequent, alternative } => {
                let cond = self.eval_bool(*condition).unwrap();
                if cond {
                    self.eval(*consequent)
                } else {
                    self.eval(*alternative)
                }
            },

            AST::Print { format, arguments } => {self.eval_print(format, arguments); Value::Unit}
        }
    }
}

pub fn interpret(ast: AST) {
    let mut p = Program::new();
    p.eval(ast);
}