use std::collections::HashMap;
use crate::error::{AsmError, AsmResult};

/// Maps label names and `.equ` constants to their integer values.
#[derive(Debug, Default)]
pub struct SymbolTable {
    map: HashMap<String, i64>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a symbol. Returns an error if the name is already defined.
    pub fn define(&mut self, name: &str, value: i64, line: usize) -> AsmResult<()> {
        if self.map.contains_key(name) {
            return Err(AsmError::new(line, format!("symbol '{name}' is already defined")));
        }
        self.map.insert(name.to_owned(), value);
        Ok(())
    }

    /// Look up a symbol. Returns an error if it is not defined.
    pub fn resolve(&self, name: &str, line: usize) -> AsmResult<i64> {
        self.map
            .get(name)
            .copied()
            .ok_or_else(|| AsmError::new(line, format!("undefined symbol '{name}'")))
    }

    /// Iterate over all symbols (for map-file output).
    pub fn iter(&self) -> impl Iterator<Item = (&str, i64)> {
        self.map.iter().map(|(k, &v)| (k.as_str(), v))
    }
}
