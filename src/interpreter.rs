use crate::{ast::AST, heap::Pointer};
use crate::heap::Heap;
use std::{collections::HashMap, collections::LinkedList, mem};

#[derive(Debug)]
pub enum Error {
    VariableMissing,
}

#[derive(Clone)]
pub enum Value {
    Int(i32),
    Boolean(bool),
    Unit,
    Array{size: i32, data: *mut Pointer},
    Object(),
}

#[derive(Clone)]
struct Function {
    parameters: Vec<String>,
    body: Box<AST>,
}

struct Runtime {
    /** Represents currently active environment */
    // TODO : Merge curr_env and call_stack_env into one.
    curr_env: LinkedList<HashMap<String, Pointer> >,
    /** Acts like a call stack. When function is called,
     *  inactive environments will be stored here.
     */
    call_stack_envs: Vec<LinkedList<HashMap<String, Pointer>>>,
    functions: HashMap<String, Function>,
    heap: Heap,
}

impl Runtime {
    pub fn new() -> Self {
        let callstack:Vec<LinkedList<HashMap<String, Pointer>>> = [LinkedList::from([HashMap::new()])].to_vec();
        
        return Runtime {
            curr_env: LinkedList::new(),
            call_stack_envs: callstack,
            functions: HashMap::new(),
            heap: Heap::new(),
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

    /// Pushes new environment on top.
    fn push_env(&mut self) {
        self.curr_env.push_front(HashMap::new());
    }

    /// Pops the top-most environment.
    fn pop_env(&mut self) {
        self.curr_env.pop_front();
    }

    fn eval_function_call(&mut self, name: &String, arguments: Vec<Box<AST>>) -> Pointer {
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

    /// Returns mutable reference to var with 'name' from environments if it exists, 
    /// otherwise returns None.
    /// Scouts the environments from the most recent one.
    fn fetch_var_mut(&mut self, name: &String) -> Result<&mut Pointer, Error> {
        // TODO : Clean this piece of code.
        for env in self.curr_env.iter_mut() {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var= env.get_mut(name);
            if var.is_some() {
                let ptr = var.unwrap();
                return Ok(ptr);
            }
        }
        // Search global env.
        for env in self.call_stack_envs.last_mut().expect("Call stack should always have global env.") {
            let ptr= env.get_mut(name);
            if ptr.is_some() {
                return Ok(ptr.unwrap());
            }
        }
        Err(Error::VariableMissing)
    }

    fn assign_to_var(&mut self, name: &String, val: Pointer) {
        let mut_var = self.fetch_var_mut(name).unwrap(); 
        *mut_var = val
    }

    /// Returns reference to var with 'name' from environments if it exists, 
    /// otherwise returns None.
    /// Scouts the environments from the most recent one.
    fn fetch_var(&mut self, name: &String) -> Result<Pointer, Error> {
        // TODO : Clean this piece of code.
        for env in self.curr_env.iter_mut() {
            /* TODO: There might be some rust magic to write this more cleanly */
            let var= env.get_mut(name);
            if var.is_some() {
                let ptr = var.unwrap();
                return Ok(*ptr);
            }
        }
        // Search global env.
        for env in self.call_stack_envs.last_mut().expect("Call stack should always have global env.") {
            let ptr= env.get_mut(name);
            if ptr.is_some() {
                return Ok(*ptr.unwrap());
            }
        }
        Err(Error::VariableMissing)
    }

    /// Adds variable to the top-most environment.
    fn add_var(&mut self, name: String, val: Pointer) -> Pointer {
        let top = self.curr_env.front_mut().expect("Missing top frame of environment.");
        /* TODO: This probably can be done with expect.
        I was however unable to do it at the time of writing this. */
        match top.insert(name, val) {
            None => val,
            /* TODO: Make the error message print the variable name.
                     If this was ever an recoverable error we need
                     to free the memory here. */
            Some(_) => panic!("Variable was redeclared."),
        }
    }

    /// Evaluates AST node as boolean, if the AST node is not boolean then return Err. 
    /// 
    fn eval_bool(&mut self, expr: AST) -> bool {
        let bool_ptr = self.eval(expr);
        let bool_val = self.heap.deref(bool_ptr);
        match bool_val {
            Value::Boolean(t) => *t,
            Value::Unit => false,
            _ => true,
        }
    }

    /// Evaluates print expression.
    fn eval_print(&mut self, format: String, arguments: Vec<Box<AST>>) {
        let mut vec_it = arguments.into_iter();

        let str: String = format.chars().map(|c| {
            match c {
                '~' => {
                    let val = *vec_it.next().expect("Expected more arguments for formatting string.");
                    let evaled_ptr = self.eval(val);
                    match self.heap.deref(evaled_ptr) {
                        Value::Int(val) => val.to_string(),
                        Value::Boolean(val) => val.to_string(),
                        Value::Unit => String::from("null"),
                        Value::Array{size: size, data: ptr} => todo!(),
                        Value::Object() => todo!(),
                        }
                    },
                _ => c.to_string(),
            }
        }).collect();
        // TODO: Quick hack to make newlines work. 
        print!("{}", str.replace("\\n", "\n"));
    }

    fn eval_top(&mut self, stmts: Vec<Box<AST>>) -> Pointer {
        let mut return_val = self.heap.get_int(0);
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

    fn eval_array(&mut self, size: Box<AST>, init: Box<AST>) -> Pointer {
        let size_ptr = self.eval(*size);
        let size = match self.heap.deref(size_ptr) {
            Value::Int(val) => *val, 
            _ => panic!("Array size needs to be an integer.")
        };
        let init = self.eval(*init);
        self.heap.alloc_array(size, init)
    }

    pub fn eval(&mut self, ast: AST) -> Pointer {
        match ast {
            AST::Integer(val) => {
                self.heap.get_int(val)
            }
          
            AST::Boolean(val) => self.heap.get_bool(val),
            
            AST::Null => self.heap.get_unit(),
         
            AST::Variable { name, value } => {
                let evaluated_val = self.eval(*value);
                self.add_var(name, evaluated_val);
                evaluated_val
            },
           
            AST::Array { size, value } => self.eval_array(size, value),
            AST::Object { extends, members } => todo!(),
            AST::AccessVariable { name } => {
                self.fetch_var(&name).expect("Variable has not been declared.")
            },
            AST::AccessField { object, field } => todo!(),
            AST::AccessArray { array, index } => {
                let array_ptr = self.eval(*array);
                let index_ptr = self.eval(*index);
                let (size, data) = match self.heap.deref(array_ptr) {
                    Value::Array{size,data} => (*size, *data),
                    _ => panic!("Can only indexate arrays.")
                };
                let index_val = self.heap.deref(index_ptr).clone();
                match index_val {
                    Value::Int(index) => {
                        if index >= size {
                            panic!("Index out of bounds!");
                        }
                        self.heap.access_array(data, index)
                    },
                    _ => panic!("Can't index array with non-int type.")
                }
            },
         
            AST::AssignVariable { name, value } => {
                let evaluated = self.eval(*value);
                self.assign_to_var(&name, evaluated);
                evaluated
            },

            AST::AssignField { object, field, value } => todo!(),
            AST::AssignArray { array, index, value } => {
                self.eval_assign_array(array, index, value)
            }
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
                let mut last_val: Option<Pointer> = None;
                self.push_env();
                for expr in exprs {
                    last_val = Some(self.eval(*expr));
                }
                self.pop_env();
                match last_val {
                    Some(val) => val,
                    None => self.heap.get_unit(),
                }
            },

            AST::Loop { condition, body } => {
                self.push_env();
                loop {
                    if !self.eval_bool((*condition).clone()) {
                        break;
                    }
                    self.eval(*body.clone());
                }
                self.pop_env();
                self.heap.get_unit()
            },

            AST::Conditional { condition, consequent, alternative } => {
                let cond = self.eval_bool(*condition);
                if cond {
                    self.eval(*consequent)
                } else {
                    self.eval(*alternative)
                }
            },

            AST::Print { format, arguments } => {
                self.eval_print(format, arguments); 
                self.heap.get_unit()
            }
        }
    }

    fn eval_assign_array(&mut self, array: Box<AST>, index: Box<AST>, value: Box<AST>) -> Pointer {
        let ptr_array = self.eval(*array);
        let ptr_index = self.eval(*index);
        let ptr_value = self.eval(*value);

        let int_index = match self.heap.deref(ptr_index) {
            Value::Int(i) => *i,
            _ => panic!("Arrays can only be indexed by integer."),
        };

        let (size, data) = match self.heap.deref(ptr_array) {
            Value::Array{size, data} => (size, *data),
            _ => panic!("Only arrays can be indexated."),
        };
        self.heap.assign_array(data, int_index, ptr_value);
        self.heap.get_unit()
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
        let int_1 = program.heap.get_int(1);
        let int_2 = program.heap.get_int(2);
        let int_3 = program.heap.get_int(3);
        let int_10 = program.heap.get_int(10);
        let int_20 = program.heap.get_int(20);
        program.add_var(String::from("x"), int_1);
        program.add_var(String::from("y"), int_2);
        program.add_var(String::from("z"), int_3);
        
        let var_x = program.fetch_var(&String::from("x")).unwrap();
        let var_y = program.fetch_var(&String::from("y")).unwrap();
        let var_z = program.fetch_var(&String::from("z")).unwrap();
        assert!(std::matches!(program.heap.deref(var_x), Value::Int(1)));
        assert!(std::matches!(program.heap.deref(var_y), Value::Int(2)));
        assert!(std::matches!(program.heap.deref(var_z), Value::Int(3)));
        assert!(std::matches!(program.fetch_var(&String::from("a")), Err(Error::VariableMissing)));

        program.push_env();
        program.add_var(String::from("x"), int_10);
        program.add_var(String::from("y"), int_20);
        
        let var_x = program.fetch_var(&String::from("x")).unwrap();
        let var_y = program.fetch_var(&String::from("y")).unwrap();
        assert!(std::matches!(program.heap.deref(var_x), Value::Int(10)));
        assert!(std::matches!(program.heap.deref(var_y), Value::Int(20)));
        assert!(std::matches!(program.heap.deref(var_z), Value::Int(3)));
        assert!(std::matches!(program.fetch_var(&String::from("a")), Err(Error::VariableMissing)));
        
        program.pop_env();
        let var_x = program.fetch_var(&String::from("x")).unwrap();
        let var_y = program.fetch_var(&String::from("y")).unwrap();
        let var_z = program.fetch_var(&String::from("z")).unwrap();
        assert!(std::matches!(program.heap.deref(var_x), Value::Int(1)));
        assert!(std::matches!(program.heap.deref(var_y), Value::Int(2)));
        assert!(std::matches!(program.heap.deref(var_z), Value::Int(3)));
        assert!(std::matches!(program.fetch_var(&String::from("a")), Err(Error::VariableMissing)));
    }

    #[test]
    fn literals() {
        let mut program = Runtime::new();

        let val1 = program.eval(AST::Integer(5));
        let val2 = program.eval(AST::Boolean(true));
        let val3 = program.eval(AST::Null);

        assert!(std::matches!(program.heap.deref(val1), Value::Int(5)));
        assert!(std::matches!(program.heap.deref(val2), Value::Boolean(true)));
        assert!(std::matches!(program.heap.deref(val3), Value::Unit));
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

        let evaled_true = program.eval(val_true);
        let evaled_false = program.eval(val_false);

        assert!(std::matches!(program.heap.deref(evaled_true), Value::Int(1)));
        assert!(std::matches!(program.heap.deref(evaled_false), Value::Int(2)));
    }

    #[test]
    fn compound() {
        let mut program = Runtime::new();

        let compound = AST::Block([AST::Integer(1).into_boxed(), AST::Integer(2).into_boxed()].to_vec());

        let evaled = program.eval(compound);
        assert!(std::matches!(program.heap.deref(evaled), Value::Int(2)));
    }

    #[test]
    fn var_assign() {
        let mut program = Runtime::new();
        program.push_env();
        let decl = AST::Variable{name: String::from("a"), value: AST::Integer(5).into_boxed()};
        program.eval(decl);
        let a = program.fetch_var(&String::from("a")).unwrap();
        assert!(std::matches!(program.heap.deref(a), Value::Int(5)));
        
        let assign = AST::AssignVariable{name: String::from("a"), value: AST::Integer(10).into_boxed()};
        program.eval(assign);
        
        let a = program.fetch_var(&String::from("a")).unwrap();
        assert!(std::matches!(program.heap.deref(a), Value::Int(10)));

        // Test that the variable 'a' will remain the same after coming from a block
        let block = AST::Block([
            AST::Variable{name: String::from("a"), value: AST::Integer(3).into_boxed()}.into_boxed(),
            AST::AssignVariable{name: String::from("a"), value: AST::Integer(2).into_boxed()}.into_boxed(),
            AST::AccessVariable{name: String::from("a")}.into_boxed(),
        ].to_vec());

        // Check that the block will return the new value of variable
        let evaled_block = program.eval(block);
        assert!(std::matches!(program.heap.deref(evaled_block), Value::Int(2)));

        // Check that the variable outside the scope retained it's value
        let a = program.fetch_var(&String::from("a")).unwrap();
        assert!(std::matches!(program.heap.deref(a), Value::Int(10)));

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

    #[test]
    fn arrays() {
        let decl = AST::Top([
            AST::Variable{name: String::from("arr"), value: AST::Array{size: AST::Integer(5).into_boxed(), value: AST::Integer(2).into_boxed()}.into_boxed()}.into_boxed(),
            AST::AssignArray{array: AST::AccessVariable{name: String::from("arr")}.into_boxed(), index: AST::Integer(1).into_boxed(), value: AST::Integer(3).into_boxed()}.into_boxed(),
        ].to_vec());
        let mut program = Runtime::new();
        program.push_env();
        program.eval(decl);
        let res0 = program.eval(AST::AccessArray{array: AST::AccessVariable{name: String::from("arr")}.into_boxed(), index: AST::Integer(0).into_boxed()});
        let res1 = program.eval(AST::AccessArray{array: AST::AccessVariable{name: String::from("arr")}.into_boxed(), index: AST::Integer(1).into_boxed()});
        let res2 = program.eval(AST::AccessArray{array: AST::AccessVariable{name: String::from("arr")}.into_boxed(), index: AST::Integer(2).into_boxed()});

        assert!(std::matches!(program.heap.deref(res0), Value::Int(2)));
        assert!(std::matches!(program.heap.deref(res1), Value::Int(3)));
        assert!(std::matches!(program.heap.deref(res2), Value::Int(2)));
    }
}
