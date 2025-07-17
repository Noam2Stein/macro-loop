use std::collections::HashMap;

use proc_macro2::Span;
use syn::{Error, Ident};

use super::value::*;

pub struct Namespace<'p> {
    parent: Option<&'p Namespace<'p>>,
    names: HashMap<String, Value>,
    new_names: HashMap<String, Value>,
}

impl<'p> Namespace<'p> {
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

    pub fn insert(&mut self, key: Ident, value: Value) -> syn::Result<()> {
        match self.new_names.insert(key.to_string(), value) {
            None => Ok(()),
            Some(_) => Err(Error::new(key.span(), format!("duplicate name {key}"))),
        }
    }
    pub fn flush(&mut self) {
        for (k, v) in self.new_names.drain() {
            self.names.insert(k, v);
        }
    }

    pub fn get(&self, key: &Ident) -> syn::Result<&Value> {
        let key_string = key.to_string();

        self.get_from_string(&key_string, key.span())
    }
}

impl<'p> Namespace<'p> {
    fn get_from_string(&self, key: &String, span: Span) -> syn::Result<&Value> {
        if let Some(value) = self.new_names.get(key) {
            Ok(value)
        } else if let Some(value) = self.names.get(key) {
            Ok(value)
        } else if let Some(parent) = self.parent {
            parent.get_from_string(key, span)
        } else {
            Err(Error::new(span, format!("cannot find {key}")))
        }
    }
}
