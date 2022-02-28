
fn TwoOp(name: String, arguments: Vec<Value>) -> Option<Value> {
    if(arguments.len() != 2) {
        panic!("Function {} requires two arguments.", name)
    }
    match (arguments[0], arguments[1]) {
        (Int(x), Int(y)) => {
            match name {
                "+" => Some(Value::Int(x + y)),
                "-" => Some(Value::Int(x - y)),
                "*" => Some(Value::Int(x * y)),
                "/" => Some(Value::Int(x / y)),
                ">" => Some(Value::Int(x > y)),
                "<" => Some(Value::Int(x < y)),
                "<=" => Some(Value::Int(x <= y)),
                ">=" => Some(Value::Int(x >= y)),
                "%" => Some(Value::Int(x % y)),
                "==" => Some(Value::Int(x == y)),
                "!=" => Some(Value::Int(x != y)),
                "&" => Some(Value::Int(x & y)),
                "|" => Some(Value::Int(x | y)),
                _ => None,
            }
        }
        (Bool(x), Bool(y)) => {
            match name {
                "&&" => Some(Value::Bool(x && y)),
                "||" => Some(Value::Bool(x || y)),
                _ => None
            }
        }
        _ => None
    }
}
