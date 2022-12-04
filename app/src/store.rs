use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct Store {
    core: Rc<RefCell<HashMap<Vec<u8>, Vec<u8>>>>,
    prefix: Vec<u8>,
}

impl Store {
    pub fn new() -> Self {
        let core = HashMap::new();
        return Store {
            core: Rc::new(RefCell::new(core)),
            prefix: vec![],
        };
    }

    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        self.core.borrow().get(&full_key).map(Clone::clone)
    }

    pub fn set(&self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
        self.core.borrow_mut().insert(k, v)
    }

    pub fn get_state_hash() -> Vec<u8> {
        return vec![];
    }

    pub fn get_sub_store(&self, prefix: &[u8]) -> Self {
        return Store {
            core: self.core.clone(),
            prefix: prefix.clone().into(),
        };
    }
}
