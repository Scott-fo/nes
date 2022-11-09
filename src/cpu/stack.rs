use super::{CPU, STACK, Memory};

pub trait Stack {
    fn stack_pop(&mut self) -> u8;
    fn stack_pop_u16(&mut self) -> u16;
    fn stack_push_u16(&mut self, data:u16);
    fn stack_push(&mut self, data: u8);
}

impl Stack for CPU {
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop();
        let hi = self.stack_pop();
        u16::from_le_bytes([lo, hi])
    }

    fn stack_push_u16(&mut self, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        // Little end should be used first so it needs to be the one most recently added to stack
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        // Popping the stack moves up the stack so we move down when we push
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }
}
