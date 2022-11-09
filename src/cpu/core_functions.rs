use super::{CPU, CpuFlags, AddressingMode, Memory};

impl CPU {
    pub fn add_to_register_a(&mut self, value: u8) {
        let sum = self.register_a as u16
                + value as u16 // Add contents of memory location to register a
                + (if self.status.contains(CpuFlags::CARRY) { // If status contains the carry bit (& if it
                                                       // was set to 1 already then the result will
                                                       // not be 0) then add 1
                    1
                } else {
                    0 // Otherwise if the carry bit was not set, then we don't need to add 1
                }) as u16;

        if sum > 0b1111_1111 {
            self.status.insert(CpuFlags::CARRY); // Insert the carry bit
        } else {
            self.status.remove(CpuFlags::CARRY); // Remove the carry bit
        }

        let result = sum as u8;

        // ^ is XOR - 1 if only 1 of the values is 1. 1 ^ 1 = 0
        // Overflow value is set if the sign bit is incorrect
        // We decide whether to set this if the result is not a valid 2's complement result
        // 0b1000_0000 is just the negative bit (RHS)
        // If LHS & RHS != 0 then that means that the negative bit was 1 in LHS
        // LHS: See if the negative bit has been incorrectly set. IE +ve + +ve = -ve
        // 1 & 1 is 1, so 1 & 1 again is not zero (& 0b1000_0000 forces all others to 0)
        // Then we know that it has overflowed because result is negative but the value we added
        // and the value we added it to (register_a) are both positive
        let value_vs_result = value ^ result; // Compare initial value vs the result
        let a_vs_result = self.register_a ^ result; // Compare result and register a (register is
                                                  // what we added value to)
        if value_vs_result & a_vs_result & CpuFlags::NEGATIVE.bits != 0 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }

        // Update the register a value with result
        self.set_register_a(result);
    }

    pub fn update_zero_and_negative_flags(&mut self, param: u8) {
        if param == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        self.update_negative_flags(param);
    }

    pub fn update_negative_flags(&mut self, param: u8) {
        // If bit 7 of A is set, then we need to set the negative value
        // AND with a value where only bit 7 is set
        // This sets all values to zero apart from bit 7 IF it was already set to 1,
        // otherwise it will also return 0
        // If bit 7 is not set, then this will return 0
        if param >> 7 == 1 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    pub fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn set_register_x(&mut self, value: u8) {
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn set_register_y(&mut self, value: u8) {
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);

                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }
            AddressingMode::IndirectY => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);

                let deref_base = u16::from_le_bytes([lo, hi]);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::NonAddressing => panic!("Mode {:?} is not supported", mode),
        }
    }
}
