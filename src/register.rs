use std::{collections::HashMap, slice::Iter};

pub type RegId = String;
pub type RegContents = Vec<char>;
pub type Registers = HashMap<RegId, RegContents>;

pub struct RegisterHandler {
    registers: Registers,
    pub current_register: RegId,
}

impl RegisterHandler {
    pub fn new() -> Self {
        RegisterHandler {
            registers: HashMap::new(),
            current_register: String::from("0"),
        }
    }

    pub fn init_reg(&mut self, reg: impl ToString, value: RegContents) {
        self.registers.entry(reg.to_string()).or_insert(value);
    }

    pub fn set_reg(&mut self, value: RegContents) {
        self.registers.insert(self.current_register.clone(), value);
    }

    pub fn push_reg(&mut self, append_value: &[char]) {
        match self.registers.get_mut(&self.current_register) {
            Some(value) => {
                value.reserve(append_value.len());
                append_value.iter().for_each(|c| {
                    value.push(*c);
                });
            }
            None => {
                self.registers
                    .insert(self.current_register.clone(), append_value.to_vec());
            }
        }
    }

    pub fn push_char(&mut self, append_value: char) {
        match self.registers.get_mut(&self.current_register) {
            Some(value) => {
                value.push(append_value);
            }
            None => {
                self.registers
                    .insert(self.current_register.clone(), vec![append_value]);
            }
        }
    }

    pub fn get_reg(&mut self) -> RegContents {
        self.registers
            .get(&self.current_register)
            .unwrap_or(&Vec::new())
            .clone()
    }

    pub fn reset_current_register(&mut self) {
        self.current_register = "0".into()
    }
}
