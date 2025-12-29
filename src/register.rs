use std::collections::HashMap;

pub type RegId = String;
pub type RegContents = String;
pub type Registers = HashMap<RegId, RegContents>;

#[derive(Clone)]
pub struct RegisterHandler {
    registers: HashMap<RegId, RegContents>,
    pub current_register: RegId,
}

impl RegisterHandler {
    pub fn new() -> Self {
        RegisterHandler {
            registers: HashMap::new(),
            current_register: String::from("0"),
        }
    }

    pub fn init_reg(&mut self, reg: impl ToString, value: impl ToString) {
        self.current_register = reg.to_string();
        self.set_reg(value.to_string());
    }

    pub fn set_reg(&mut self, value: RegContents) {
        self.registers.insert(self.current_register.clone(), value);
    }

    pub fn empty_reg(&mut self) {
        self.registers
            .insert(self.current_register.clone(), String::new());
    }

    pub fn push_reg(&mut self, append_value: &str) {
        match self.registers.get_mut(&self.current_register) {
            Some(value) => {
                value.reserve(append_value.len());
                value.push_str(append_value);
            }
            None => {
                self.registers
                    .insert(self.current_register.clone(), append_value.to_string());
            }
        }
    }

    pub fn get_reg(&mut self) -> &str {
        match self.registers.get(&self.current_register) {
            Some(s) => s,
            None => "",
        }
    }

    pub fn reset_current_register(&mut self) {
        self.current_register = "0".into()
    }
}
