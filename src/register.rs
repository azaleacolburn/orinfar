use std::collections::HashMap;

pub type RegId = String;
pub type RegContents = String;
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

    pub fn init_reg(&mut self, reg: impl ToString, value: impl ToString) {
        self.registers
            .entry(reg.to_string())
            .or_insert(value.to_string());
    }
}
