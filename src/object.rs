use std::collections::HashMap;
use std::fmt::Debug;

pub trait Object: Debug {
    fn send(&mut self, pool: &mut ObjectPool, target: ObjectRef, message: EvaluatedMessage) -> ObjectRef;
    fn define(&mut self, name: String, value: ObjectRef) -> ObjectRef;
}

// A Message where all the expressions for the passed parametser
// have been evaluated into object refrences.
#[derive(Debug)]
pub struct EvaluatedMessage {
    name: String,
    arguments: Vec<(String, ObjectRef)>
}

#[derive(Debug)]
pub struct NormalObject {
    prototype: ObjectRef,
    properties: HashMap<String, ObjectRef>,
    metadata: Metadata
}

impl NormalObject {
    fn extending(prototype: ObjectRef) -> NormalObject {
        NormalObject{
            prototype: prototype,
            methods: HashMap::new(),
            properties: HashMap::new()
        }
    }
}

fn get_handler(target: Object) {
}

impl NormalObject {
    fn get_handler(&mut self, ObjectRef, message: EvaluatedMessage) -> ObjectRef {
        match self.properties.get_mut(&message.name) {
            Some(defined_object) => {
                let clone = defined_object.clone();
                let message = EvaluatedMessage {name: "call".into(), arguments: Vec::new()};
                defined_object.send(clone, message)
            }
            None => self.prototype.send(target, message)
        }
    }

    fn define(&mut self, name: String, value: ObjectRef) -> ObjectRef {
        self.properties.insert(name, value.clone());
        value.clone()
    }
}


#[derive(Debug, Clone)]
pub enum Metadata {
    NumericValue(i64),
    None
}

pub struct ObjectPool {
    normal_objects: Vec<NormalObject>,
    special_objects: Vec<Box<Object>>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectRef {
    Normal(usize),
    Special(usize)
}

impl ObjectPool {
    fn create(&mut self, prototype: ObjectRef) -> ObjectRef {
        let object = NormalObject::extending(prototype);
        self.normal_objects.push(object);
        ObjectRef::Normal(self.normal_objects.len() - 1)
    }

    // Returns the callable object that will handle a response
    fn send(&mut self, reference: ObjectRef, message: EvaluatedMessage) -> ObjectRef {
        match reference {
            ObjectRef::Normal(index) => self.normal_objects[index].send(self, message),
            ObjectRef::Special(index) => self.special_objects[index].send(self, message)
        }
    }

    fn define(&mut self, reference: ObjectRef, name: String, value: ObjectRef) -> ObjectRef {
    }
}