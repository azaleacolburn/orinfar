use std::{collections::HashMap, fmt::Display};

pub type RegId = char;
pub type RegContents = String;
pub type Registers = HashMap<RegId, RegContents>;

#[derive(Clone)]
pub struct RegisterHandler {
    registers: Registers,
    pub current_register: RegId,
}

impl RegisterHandler {
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            current_register: 'a',
        }
    }

    pub fn init_reg(&mut self, reg: char, value: &impl ToString) {
        self.current_register = reg;
        self.set_reg(value.to_string());
    }

    pub fn set_reg(&mut self, value: RegContents) {
        self.registers.insert(self.current_register, value);
    }

    pub fn empty_reg(&mut self) {
        self.registers.insert(self.current_register, String::new());
    }

    pub fn push_reg(&mut self, append_value: &impl ToString) {
        let str = append_value.to_string();
        match self.registers.get_mut(&self.current_register) {
            Some(value) => {
                value.push_str(&str);
            }
            None => {
                self.registers
                    .insert(self.current_register, append_value.to_string());
            }
        }
    }

    pub fn get_reg(&self) -> &str {
        match self.registers.get(&self.current_register) {
            Some(s) => s,
            None => "",
        }
    }

    pub const fn get_curr_reg(&self) -> char {
        self.current_register
    }

    pub const fn reset_curr_register(&mut self) {
        self.current_register = 'a';
    }
}

impl Default for RegisterHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for RegisterHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .registers
            .iter()
            .map(|(name, contents)| {
                if *name == self.current_register {
                    format!("(*) {name}: '{contents}'\n")
                } else {
                    format!("{name}: '{contents}'\n")
                }
            })
            .collect::<String>();

        f.write_str(&str)
    }
}
