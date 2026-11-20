use super::*;

impl Nominals {
    pub fn arguments(&self, name: &str) -> Vec<Argument> {
        self.arguments
            .borrow()
            .get(name)
            .cloned()
            .unwrap_or_default()
    }
}

impl Nominals {
    pub fn origin(&self, name: &str) -> String {
        self.origins
            .borrow()
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_owned())
    }
}

impl Nominals {
    pub fn get(&self, name: &str) -> Option<Type> {
        self.instances.borrow().get(name).cloned()
    }
}

impl Nominals {
    pub fn contains(&self, name: &str) -> bool {
        self.definitions.contains_key(name) || self.instances.borrow().contains_key(name)
    }
}
