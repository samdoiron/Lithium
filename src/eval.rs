use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;

use parser::{Target, Message, Expression, Block, Statement};
use object::{Object, ObjectRef, ObjectPool, Metadata};

#[derive(Debug)]
struct Signature {
    parameters: Vec<String>
}

impl Signature {
    fn is_message_valid(&self, message: &EvaluatedMessage) -> bool {
        if message.arguments.len() != self.parameters.len() { return false; }
        for parameter in &self.parameters {
            if !message.arguments.iter().any(|&(ref name, _)| name == parameter) {
                return false
            }
        }
        true
    }
}

#[derive(Debug)]
struct Lambda {
    parent_scope: ObjectRef,
    body: Box<Block>
}

impl Lambda {
    fn call_with_captured_context(&self, program: &mut Program) -> ObjectRef {
        eval_block(pool, self.parent_scope, &self.body)
    }

    // Context given, such as when we are evaluating a method that has bubbled
    // up from a child object.
    fn call_with_context(&self, context: ObjectRef) -> ObjectRef {
        eval_block(context, &self.body)
    }
}

impl Object for Lambda {
    fn send(&mut self, pool: &mut ObjectPool, _target: ObjectRef, message: EvaluatedMessage) -> ObjectRef {
        match message.name {
            ref m if m == "call" => self.call_with_captured_context(pool),
            _ => panic!("Unknown message sent to lambda")
        }
    }

    fn define(&mut self, _name: String, _value: ObjectRef) -> ObjectRef {
        panic!("Cannot extend native object Lambda");
    }
}

impl ObjectRef {
    fn new(object: Box<Object>) -> ObjectRef {
        ObjectRef{
            object: Rc::new(RefCell::new(object)),
            metadata: Metadata::None
        }
    }

    fn new_with_metadata(object: Box<Object>, metadata: Metadata) -> ObjectRef {
        ObjectRef{
            object: Rc::new(RefCell::new(object)),
            metadata: metadata
        }
    }
}

impl Object for ObjectRef {
    fn send(&mut self, target: ObjectRef, message: EvaluatedMessage) -> ObjectRef {
        self.object.borrow_mut().send(target, message)
    }

    fn define(&mut self, name: String, value: ObjectRef) -> ObjectRef {
        self.object.borrow_mut().define(name, value)
    }
}

#[derive(Debug)]
struct RootObject { }

impl Object for RootObject {
    fn send(&mut self, _target: ObjectRef, message: EvaluatedMessage) -> ObjectRef {
        match message {
            _ => panic!("Unknown root message")
        }
    }

    fn define(&mut self, _name: String, _value: ObjectRef) -> ObjectRef {
        panic!("Attempt to define on the root scope. This is Evil, cut it out.")
    }
}

#[derive(Debug, Clone)]
struct Void { } 

impl Void {
    fn new_reference() -> ObjectRef {
        ObjectRef::new(Box::new(Void{}))
    }
}

impl Object for Void {
    fn send(&mut self, target: ObjectRef, _message: EvaluatedMessage) -> ObjectRef {
        target.clone()
    }

    fn define(&mut self, _name: String, _value: ObjectRef) -> ObjectRef {
        panic!("You have stared into the void");
    }
}

#[derive(Debug)]
struct Number { }

impl Number {
    fn new_reference(digits: &str) -> ObjectRef {
        Number::new_from_value(digits.parse::<i64>().unwrap())
    }

    fn new_from_value(value: i64) -> ObjectRef {
        ObjectRef::new_with_metadata(
            Box::new(Number{}),
            Metadata::NumericValue(value)
        )
    }
}

fn get_argument(target: &str, arguments: Vec<(String, ObjectRef)>) -> ObjectRef {
    arguments.iter().filter(|&&(ref name, _)| name == target).map(|&(_, ref value)| value.clone())
        .next().unwrap()
}

impl Object for Number {
    fn send(&mut self, target: ObjectRef, message: EvaluatedMessage) -> ObjectRef {
        let numeric_value = match target.metadata {
            Metadata::NumericValue(val) => val,
            _ => panic!("Number type has no numeric value metadata")
        };

        let number_add_signature: Signature = Signature {
            parameters: vec!["to".to_string()]
        };

        match &message.name {
            m if m == "println" => {
                println!("{}", numeric_value);
                Void::new_reference()
            },
            m if m == "add" => {
                if !number_add_signature.is_message_valid(&message) {
                    panic!("Invalid signature for Number#add")
                }
                let other = get_argument("to", message.arguments);
                let sum = match other.metadata {
                    Metadata::NumericValue(val) => numeric_value + val,
                    _ => panic!("Number#add must be called with a number")
                };
                Number::new_from_value(sum)
            }
            _ => { panic!("Because it got that way") }
        }
    }

    fn define(&mut self, _name: String, _value: ObjectRef) -> ObjectRef {
        panic!("Cannot extend native object Number");
    }
}

pub struct Program {
    pool: ObjectPool
}

impl Program {
    pub fn eval(&mut self, block: Block) {
        let root = Box::new(RootObject{});
        let root_ref = ObjectRef::new(root);
        self.eval_block(root_ref, &block);
    }

    fn eval_block(&mut self, parent_scope: ObjectRef, block: &Block) -> ObjectRef {
        let scope = ObjectRef::new(Box::new(NormalObject::extending(parent_scope)));
        let mut statements = block.statements.iter();
        let mut last = self.eval_statement(scope, statements.next().expect("Cannot evaluate empty block"));
        for statement in statements {
            last = self.eval_statement(scope, statement);
        }
    }

    fn eval_statement(&mut self, mut scope: ObjectRef, statement: &Statement) -> ObjectRef {
        match statement {
            &Statement::Definition(ref definition) => {
                let value = self.eval_expression(scope, &definition.value);
                scope.define(definition.target.clone(), value)
            }
            &Statement::Expression(ref expression) => self.eval_expression(scope, &expression)
        }
    }

    fn eval_message(&mut self, scope: ObjectRef, message: &Message) -> EvaluatedMessage {
        let bindings = message.arguments.iter().map(|arg| {
            let evaluated = self.eval_expression(scope, &arg.value);
            (arg.name.clone(), evaluated)
        }).collect();
        EvaluatedMessage { name: message.name.clone(), arguments: bindings }
    }

    fn eval_expression(&mut self, mut scope: ObjectRef, expression: &Expression) -> ObjectRef {
        match expression {
            &Expression::Send(ref send) => {
                let mut target = match &send.target {
                    &Target::Identifier(ref ident) => {
                        let message = EvaluatedMessage{name: ident.to_string(), arguments: Vec::new() };
                        scope.send(scope, message)
                    },
                    &Target::Number(ref num) => Number::new_reference(num),
                    &Target::Expression(ref target_expression) => {
                        self.eval_expression(scope, target_expression)
                    }
                };
                send.messages.iter().map(|message| {
                    let target_clone = target.clone();
                    target.send(target_clone, self.eval_message(scope, &message))
                }).last().expect("Uh oh, cannot determine the value of an empty expression")
            },
            &Expression::Number(ref digits) => Number::new_reference(digits),
            &Expression::Lambda(ref block) => {
                ObjectRef::new(Box::new(Lambda{parent_scope: scope.clone(), body: block.clone()}))
            }
        }
    }
}
