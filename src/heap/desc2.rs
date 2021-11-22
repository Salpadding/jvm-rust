pub enum FieldDescriptor {
    Class(String),
    Array {
        dims: usize,
        component: Box<FieldDescriptor>,
    },
    Primitive(char),
    Void,
}

pub struct MethodDescriptor {
    params: Vec<FieldDescriptor>,
    ret: FieldDescriptor,
    arg_slots: usize,
}
