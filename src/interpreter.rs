use crate::ast::AST;
use std::{collections::HashMap, collections::LinkedList, mem};

#[derive(Copy, Clone)]
enum Value {
    Int(i32),
    Boolean(bool),
    Unit,
}

#[derive(Clone)]
struct Function {
    parameters: Vec<String>,
    body: Box<AST>,
}

struct Runtime {
    /** Represents currently active environment */
    // TODO : Merge curr_env and call_stack_env into one.
    curr_env: LinkedList<HashMap<String, Value> >,
    /** Acts like a call stack. When function is called,
     *  inactive environments will be stored here.
     */
    call_stack_envs: Vec<LinkedList<HashMap<String, Value>>>,
    functions: HashMap<String, Function>,
}

impl Runtime {
    pub fn new() -> Self {
        let callstck:Vec<LinkedList<HashMap<String, Value>>> = [LinkedList::from([HashMap::new()])].to_vec();
        
        return Runtime {
            curr_env: LinkedList::new(),
            call_stack_envs: callstck,
            functions: HashMap::new(),
        }
    }

    /// Saves the current environment to the top of the call stack and
    /// pushes new environment to the current.
    fn save_env(&mut self) {
        self.call_stack_envs.push(mem::replace(&mut self.curr_env, LinkedList::new()));
        self.push_env();
    }

    /// Restores top-most environment from the call stack and dumps the
    /// current environments.
    fn restore_env(&mut self) {
        self.curr_env = self.call_stack_envs.pop().expect("Can't restore non-existing environment.");
    }

    fn add_function(&mut self, name: String, parameters: Vec<String>, body: Box<AST>) {
        self.functions.insert(name, Function{parameters, body});
    }

    fn eval_function_call(&mut self, name: &String, arguments: Vec<Box<AST>>) -> Value {
        self.save_env();
        let function: Function = self.functions.get(name).expect("Called function is not defined.").clone();
        if function.parameters.len() != arguments.len() {
            panic!("Wrong number of arguments in function call '{}'", name);
        }
        for (name, val) in function.parameters.into_iter().zip(arguments.into_iter()) {
            let result= self.eval(*val);
            self.add_var(name, result);
        }
        let result = self.eval(*function.body);
        self.restore_env();
        result
    }

    /// Pushes new environment on top.
    fn push_env(&mut self) {
        self.curr_env.push_front(HashMap::new());
    }

    /// Pops the top-most environment.
    fn pop_env(&mut self) {
        self.curr_env.pop_front();
    }

    /// Returns mutable reference to var with 'name' from environments if it exists, 
    /// otherwise returns None.
    /// Scouts the environments from the most recent one.
    fn fetch_var_mut(&mut self, name: &String) -> Option<&mut Value> {
        // TODO : Clean this piece of code.
        for env in self.curr_env.iter_mut() {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var= env.get_mut(name);
            if var.is_some() {
                return var;
            }
        }
        // Search global env.
        for env in self.call_stack_envs.last_mut().expect("Call stack should always have global env.") {
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
        for env in self.curr_env.iter() {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var= env.get(name);
            if var.is_some() {
                return var;
            }
        }
        // Search global env.
        for env in self.call_stack_envs.last().expect("Call stack should always have global env.") {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var = env.get(name);
            if var.is_some() {
                return var;
            }
        }
        None
    }

    /// Adds variable to the top-most environment.
    fn add_var(&mut self, name: String, val: Value) {
        let top = self.curr_env.front_mut().expect("Missing top frame of environment.");
        /* TODO: This probably can be done with expect.
        I was however unable to do it at the time of writing this. */
        match top.insert(name, val) {
            None => (),
            /* TODO: Make the error message print the variable name. */
            Some(_) => panic!("Variable was redeclared."),
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

        let str: String = format.chars().map(|c| {
            match c {
                '~' => {
                    let val = *vec_it.next().expect("Expected more arguments for formatting string.");
                    match self.eval(val) {
                        Value::Int(val) => val.to_string(),
                        Value::Boolean(val) => val.to_string(),
                        Value::Unit => String::from("null"),
                        }
                    },
                _ => c.to_string(),
            }
        }).collect();
        // TODO: Quick hack to make newlines work. 
        print!("{}", str.replace("\\n", "\n"));
    }

    fn eval_top(&mut self, stmts: Vec<Box<AST>>) -> Value {
        let mut return_val: Value = Value::Int(0);
        for stmt in stmts {
            match *stmt {
                AST::Function { name, parameters, body } => {
                    self.add_function(name, parameters, body)
                }
                _ => {
                    return_val = self.eval(*stmt);
                    ()
                }
            }
        };
        return_val
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
            AST::AccessVariable { name } => {
                *self.fetch_var(&name).expect("Variable has not been declared.")
            },
            AST::AccessField { object, field } => todo!(),
            AST::AccessArray { array, index } => todo!(),
         
            AST::AssignVariable { name, value } => {
                let evaluated = self.eval(*value);
                self.assign_to_var(&name, evaluated);
                evaluated
            },

            AST::AssignField { object, field, value } => todo!(),
            AST::AssignArray { array, index, value } => todo!(),
            AST::Function { name, parameters, body } => {
                panic!("Function can only be declared as top level statement.");
            }

            AST::CallFunction { name, arguments } => {
                self.eval_function_call(&name, arguments)
            },
            AST::CallMethod { object, name, arguments } => todo!(),
            AST::Top(exprs) => {
                self.eval_top(exprs)
            },

            AST::Block(exprs) => {
                let mut last_val: Option<Value> = None;
                self.push_env();
                for expr in exprs {
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

            AST::Print { format, arguments } => {
                self.eval_print(format, arguments); 
                Value::Unit
            }
        }
    }
}

pub fn interpret(ast: AST) {
    let mut p = Runtime::new();
    p.push_env();
    match ast {
        AST::Top(stmts) => {
            p.eval_top(stmts);
        }
        _ => panic!("Program must begin by top-level statement.")
    }
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn environment() {
        let mut program = Runtime::new();

        program.push_env();
        program.add_var(String::from("x"), Value::Int(1));
        program.add_var(String::from("y"), Value::Int(2));
        program.add_var(String::from("z"), Value::Int(3));
        
        assert!(std::matches!(program.fetch_var(&String::from("x")), Some(Value::Int(1))));
        assert!(std::matches!(program.fetch_var(&String::from("y")), Some(Value::Int(2))));
        assert!(std::matches!(program.fetch_var(&String::from("z")), Some(Value::Int(3))));
        assert!(std::matches!(program.fetch_var(&String::from("a")), None));

        program.push_env();
        program.add_var(String::from("x"), Value::Int(10));
        program.add_var(String::from("y"), Value::Int(20));
        
        assert!(std::matches!(program.fetch_var(&String::from("x")), Some(Value::Int(10))));
        assert!(std::matches!(program.fetch_var(&String::from("y")), Some(Value::Int(20))));
        assert!(std::matches!(program.fetch_var(&String::from("z")), Some(Value::Int(3))));
        assert!(std::matches!(program.fetch_var(&String::from("a")), None));

        program.pop_env();
        assert!(std::matches!(program.fetch_var(&String::from("x")), Some(Value::Int(1))));
        assert!(std::matches!(program.fetch_var(&String::from("y")), Some(Value::Int(2))));
        assert!(std::matches!(program.fetch_var(&String::from("z")), Some(Value::Int(3))));
        assert!(std::matches!(program.fetch_var(&String::from("a")), None));
    }

    #[test]
    fn literals() {
        let mut program = Runtime::new();

        let val1 = AST::Integer(5);
        let val2 = AST::Boolean(true);
        let val3 = AST::Null;

        assert!(std::matches!(program.eval(val1), Value::Int(5)));
        assert!(std::matches!(program.eval(val2), Value::Boolean(true)));
        assert!(std::matches!(program.eval(val3), Value::Unit));
    }

    #[test]
    fn conditional() {
        let mut program = Runtime::new();

        let val_true = AST::Conditional{
            condition: AST::Boolean(true).into_boxed(),
            consequent: AST::Integer(1).into_boxed(),
            alternative: AST::Integer(2).into_boxed()
        };

        let val_false = AST::Conditional{
            condition: AST::Boolean(false).into_boxed(),
            consequent: AST::Integer(1).into_boxed(),
            alternative: AST::Integer(2).into_boxed()
        };

        assert!(std::matches!(program.eval(val_true), Value::Int(1)));
        assert!(std::matches!(program.eval(val_false), Value::Int(2)));
    }

    #[test]
    fn compound() {
        let mut program = Runtime::new();

        let compound = AST::Block([AST::Integer(1).into_boxed(), AST::Integer(2).into_boxed()].to_vec());

        assert!(std::matches!(program.eval(compound), Value::Int(2)));
    }

    #[test]
    fn var_assign() {
        let mut program = Runtime::new();
        program.push_env();
        let decl = AST::Variable{name: String::from("a"), value: AST::Integer(5).into_boxed()};
        program.eval(decl);
        assert!(std::matches!(program.fetch_var(&String::from("a")), Some(Value::Int(5))));

        let assign = AST::AssignVariable{name: String::from("a"), value: AST::Integer(10).into_boxed()};
        program.eval(assign);
        assert!(std::matches!(program.fetch_var(&String::from("a")), Some(Value::Int(10))));

        // Test that the variable 'a' will remain the same after coming from a block
        let block = AST::Block([
            AST::Variable{name: String::from("a"), value: AST::Integer(3).into_boxed()}.into_boxed(),
            AST::AssignVariable{name: String::from("a"), value: AST::Integer(2).into_boxed()}.into_boxed(),
            AST::AccessVariable{name: String::from("a")}.into_boxed(),
        ].to_vec());

        // Check that the block will return the new value of variable
        assert!(std::matches!(program.eval(block), Value::Int(2)));

        // Check that the variable outside the scope retained it's value
        assert!(std::matches!(program.fetch_var(&String::from("a")), Some(Value::Int(10))));

        program.pop_env();
    }

    #[test]
    fn function_call() {
        let mut program = Runtime::new();
        program.push_env();
        let decl = 
        AST::Top([
            AST::Variable{name: String::from("x"), value: AST::Integer(3).into_boxed()}.into_boxed(),
            AST::Function{name: String::from("foo"), parameters: [String::from("y")].to_vec(),
                body: AST::AccessVariable{name: String::from("x")}.into_boxed()}.into_boxed(),
            AST::CallFunction{name: String::from("foo"), arguments: [AST::Integer(1).into_boxed()].to_vec()}.into_boxed(),
        ].to_vec());
        program.eval(decl);
    }
}
