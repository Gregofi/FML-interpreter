
use core::ffi::c_void;
use crate::interpreter::Value;
use std::collections::{HashMap};

#[derive(Copy, Clone)]
pub struct Pointer{
    data: *mut Value,
}

extern "C" {
    #[link(name="heap_init", kind="static")]
    fn heap_init();
    #[link(name="heap_alloc", kind="static")]
    fn heap_alloc(bytes: i32) -> *mut c_void;
    #[link(name="heap_free", kind="static")]
    fn heap_free(ptr: *mut c_void) -> bool;
    #[link(name="heap_done", kind="static")]
    fn heap_done() -> i32;
}

pub struct Heap {
    int_literals: HashMap<i32, Pointer>,
    bool_literals: HashMap<bool, Pointer>,
    unit: Pointer,
}

impl Heap {
    pub fn new() -> Self {
        unsafe {
            heap_init();
            let mut heap = Heap {
                int_literals: HashMap::new(),
                bool_literals: HashMap::new(),
                unit: Pointer{data: 0 as *mut Value},
                };
            heap.unit = heap.alloc(Value::Unit);
            heap
        }
    }

    pub fn drop(&mut self) {
        unsafe {
            heap_done();
        }
    }

    pub fn alloc(&mut self, value: Value) -> Pointer {
        unsafe {
            let ptr = heap_alloc(std::mem::size_of_val(&value).try_into().unwrap()) as *mut Value;
            *ptr = value;
            Pointer{data: ptr}
        }
    }

    pub fn deref(&self, ptr: Pointer) -> &Value {
        unsafe {
            &*ptr.data
        }
    }

    pub fn deref_mut(&mut self, ptr: Pointer) -> &mut Value {
        unsafe {
            &mut*ptr.data
        }
    }

    /// Returns pointer to an integer constant value on the heap.
    /// If this int isn't on the heap, allocation will be done.
    pub fn get_int(&mut self, val: i32) -> Pointer {
        let int_lit = self.int_literals.get(&val);
        match int_lit {
            Some(ptr) => *ptr,
            None =>  {
                let int_ptr = self.alloc(Value::Int(val));
                self.int_literals.insert(val, int_ptr);
                int_ptr
            }
        }
    }
    
    pub fn get_bool(&mut self, val: bool) -> Pointer {
        match self.bool_literals.get(&val) {
            Some(ptr) => *ptr,
            None =>  {
                let bool_ptr = self.alloc(Value::Boolean(val));
                self.bool_literals.insert(val, bool_ptr);
                bool_ptr
            }
        }
    }

    pub fn get_unit(&self) -> Pointer {
        self.unit
    }
}