use std::collections::HashMap;

use syn::Error;

use super::*;

pub struct Namespace<'p, 'v> {
    parent: Option<&'p Namespace<'p, 'v>>,
    names: HashMap<NameId, ValueRef<'v>>,
    new_names: HashMap<NameId, ValueRef<'v>>,
}

impl<'p, 'v> Namespace<'p, 'v> {
    pub fn new() -> Self {
        Self {
            parent: None,
            names: HashMap::new(),
            new_names: HashMap::new(),
        }
    }

    pub fn fork(&self) -> Namespace {
        Namespace {
            parent: Some(self),
            names: HashMap::new(),
            new_names: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &Name, value: ValueRef<'v>) -> syn::Result<()> {
        match self.new_names.insert(name.id().clone(), value) {
            None => Ok(()),
            Some(_) => Err(Error::new(name.span(), format!("duplicate name `{name}`"))),
        }
    }

    pub fn flush(&mut self) {
        for (k, v) in self.new_names.drain() {
            self.names.insert(k, v);
        }
    }

    pub fn get(&self, name: &Name) -> syn::Result<&Value> {
        if let Some(value) = self.new_names.get(name) {
            Ok(value)
        } else if let Some(value) = self.names.get(name) {
            Ok(value)
        } else if let Some(parent) = self.parent {
            parent.get(name)
        } else {
            Err(Error::new(name.span(), format!("cannot find {name}")))
        }
    }
}
