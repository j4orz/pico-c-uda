use std::mem;

use super::{Node, NodeDef, NodeIdCounter, OpCode};

// types form a symmetric complete bounded (ranked) lattice
// see: https://en.wikipedia.org/wiki/Lattice_(order)

// NB1: top elements denote maybe constants.
//      middle elements denote constants.
//      bot elements denote not constants.

//      peepholes are pessimistic, assuming worst case scenario
//      by starting at the bottom of the latice until proven better
//      **for now, the only constants are integers.

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type { Bot, Top, Simple, Int(i128) } //, Tup(Vec<Box<Self>>) }
impl Type {
    pub fn is_constant(&self) -> bool {
        match self {
            Type::Bot => false, Type::Top => true,
            Type::Simple => todo!(), Type::Int(_) => true,
            // Type::Tup(type_and_vals) => todo!(),
        }
    }
}

impl NodeDef {
    pub fn peephole(self, nodeid_counter: &mut NodeIdCounter, start_node: &NodeDef ) -> NodeDef {
        self.borrow_mut().typ = self.eval_type();
        let peepholed = match (self.borrow().opcode, self.borrow().typ.is_constant()) {
            (OpCode::Con, true) | (_, false) => None,
            (_, true) => {
                let con = Node::new_constant( nodeid_counter, OpCode::Con, self.borrow().typ);
                let _ = con.add_def(start_node);
                println!("constant folded with node: {:?}", con);
                Some(con)
            },
        };
 
        // NB: explicit drop over implicit for asserting invariant: this node (and it's edges) SHOULD be droppable.
        match peepholed { Some(peeped) => { mem::drop(self); peeped } None => self }
    }

    // see: https://en.wikipedia.org/wiki/Partial_evaluation
    fn eval_type(&self) -> Type {
        match self.borrow().opcode {
            OpCode::Start => Type::Bot, OpCode::Ret => Type::Bot,
            OpCode::Con => self.borrow().typ.clone(), // con's already have static type (dynamic value)
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                let (x_type, y_type) = (self.borrow().defs[0].borrow().typ, self.borrow().defs[1].borrow().typ);
                let evald_type = match (x_type, y_type) {
                    (Type::Int(x), Type::Int(y)) => match self.borrow().opcode {
                        // partial evaluation, TODO: semantics are inherited from rust
                        OpCode::Add => Type::Int(x+y), 
                        OpCode::Sub => Type::Int(x-y),
                        OpCode::Mul => Type::Int(x*y),
                        OpCode::Div => Type::Int(x/y),
                        _ => panic!()
                    }
                    _ => Type::Bot,
                };

                evald_type
            },
            _ => unimplemented!()
        }
    }

    fn _idealize(&self) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod peephole {
    use crate::son::{parser::{lex, Parser}, NodeIdCounter, OpCode};
    use std::{assert_matches::assert_matches, fs};
    
    const TEST_DIR: &str = "tests/arith";

    #[test]
    fn add() {
        let chars = fs::read(format!("{TEST_DIR}/add.c"))
            .expect("file dne")
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();
    
        let mut nodeid_counter = NodeIdCounter::new(0);
        let mut parser = Parser::new(&mut nodeid_counter);
        let tokens = lex(&chars).unwrap();
        let graph = parser.parse(&tokens).unwrap();

        assert_matches!(graph.borrow().opcode, OpCode::Ret);
        assert_matches!(graph.borrow().defs[0].borrow().opcode, OpCode::Start);
        insta::assert_debug_snapshot!(graph);
    }

    #[test]
    fn sub() {
        let chars = fs::read(format!("{TEST_DIR}/sub.c"))
            .expect("file dne")
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();
    
        let mut nodeid_counter = NodeIdCounter::new(0);
        let mut parser = Parser::new(&mut nodeid_counter);
        let tokens = lex(&chars).unwrap();
        let graph = parser.parse(&tokens).unwrap();

        assert_matches!(graph.borrow().opcode, OpCode::Ret);
        assert_matches!(graph.borrow().defs[0].borrow().opcode, OpCode::Start);
        insta::assert_debug_snapshot!(graph);
    }

    #[test]
    fn mul() {
        let chars = fs::read(format!("{TEST_DIR}/mul.c"))
            .expect("file dne")
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();
    
        let mut nodeid_counter = NodeIdCounter::new(0);
        let mut parser = Parser::new(&mut nodeid_counter);
        let tokens = lex(&chars).unwrap();
        let graph = parser.parse(&tokens).unwrap();

        assert_matches!(graph.borrow().opcode, OpCode::Ret);
        assert_matches!(graph.borrow().defs[0].borrow().opcode, OpCode::Start);
        insta::assert_debug_snapshot!(graph);
    }

    #[test]
    fn div() {
        let chars = fs::read(format!("{TEST_DIR}/div.c"))
            .expect("file dne")
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();
    
        let mut nodeid_counter = NodeIdCounter::new(0);
        let mut parser = Parser::new(&mut nodeid_counter);
        let tokens = lex(&chars).unwrap();
        let graph = parser.parse(&tokens).unwrap();

        assert_matches!(graph.borrow().opcode, OpCode::Ret);
        assert_matches!(graph.borrow().defs[0].borrow().opcode, OpCode::Start);
        insta::assert_debug_snapshot!(graph);
    }

    #[test]
    fn add_compound() {
        let chars = fs::read(format!("{TEST_DIR}/add_compound.c"))
            .expect("file dne")
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();
    
        let mut nodeid_counter = NodeIdCounter::new(0);
        let mut parser = Parser::new(&mut nodeid_counter);
        let tokens = lex(&chars).unwrap();
        let graph = parser.parse(&tokens).unwrap();

        assert_matches!(graph.borrow().opcode, OpCode::Ret);
        assert_matches!(graph.borrow().defs[0].borrow().opcode, OpCode::Start);
        insta::assert_debug_snapshot!(graph);
    }
}
