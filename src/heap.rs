
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

impl Drop for Heap {
    fn drop(&mut self) {
        unsafe {
            heap_done();
        }
    }
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

    pub fn alloc(&mut self, value: Value) -> Pointer {
        unsafe {
            let ptr = heap_alloc(std::mem::size_of_val(&value).try_into().unwrap()) as *mut Value;
            if ptr.is_null() {
                panic!("A null pointer was returned by alloc.")
            }
            *ptr = value;
            Pointer{data: ptr}
        }
    }

    pub fn alloc_bytes(&mut self, bytes: usize) -> *mut Pointer {
        unsafe {
            let ptr =heap_alloc(bytes.try_into().expect("Couldn't convert 'bytes' to 32bit integer.")) as *mut Pointer;
            if ptr.is_null() {
                panic!("A null pointer was returned by alloc.")
            }
            ptr
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

    pub fn alloc_array(&mut self, size: i32, init_vec: Vec<Pointer>) -> Pointer {
        let ptr_data = self.alloc_bytes(std::mem::size_of::<Pointer>() * size as usize);
        unsafe {
            // Initialize all fields of array with init
            for (pos, item) in init_vec.iter().enumerate() {
                *(ptr_data.offset(pos.try_into().unwrap())) = *item;
            }
        }
        self.alloc(Value::Array{size: size, data: ptr_data})
    }

    pub fn assign_array(&mut self, array_data: *mut Pointer, index: i32, data: Pointer) {
        unsafe {
            *array_data.offset(index.try_into().unwrap()) = data
        }
    }

    pub fn access_array(&mut self, array_data: *mut Pointer, index: i32) -> Pointer {
        unsafe {
            *array_data.offset(index.try_into().unwrap())
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