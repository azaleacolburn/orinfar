use std::ops::Deref;

pub struct StatusBar {
    buffer: Vec<char>,
    idx: usize,
}

impl StatusBar {
    pub fn new() -> Self {
        StatusBar {
            buffer: vec![],
            idx: 0,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn buffer(&self) -> String {
        self.buffer.iter().collect()
    }

    pub fn push(&mut self, char: char) {
        self.buffer.push(char);
        self.idx += 1;
    }

    pub fn delete(&mut self) {
        if self.idx <= 1 {
            return;
        }

        self.buffer.remove(self.idx - 1);
        self.idx -= 1;
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.idx = 0;
    }
}

impl Deref for StatusBar {
    type Target = [char];
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
