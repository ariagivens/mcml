#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Var {
    pub name: Option<String>,
    pub id: u32,
}

pub struct VarFactory {
    next_var_id: u32,
}

impl VarFactory {
    pub fn new() -> Self {
        VarFactory { next_var_id: 0 }
    }

    pub fn tmp(&mut self) -> Var {
        Var {
            name: None,
            id: self.id(),
        }
    }

    pub fn named(&mut self, name: String) -> Var {
        Var {
            name: Some(name),
            id: self.id(),
        }
    }

    fn id(&mut self) -> u32 {
        let id = self.next_var_id;
        self.next_var_id += 1;
        id
    }
}
