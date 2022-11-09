use super::{CPU, AddressingMode, CpuFlags, Stack, Memory};

impl CPU {
    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem = self.mem_read(addr);
        // Add contents of a mem location to accumulator with carry bit. If overflow, set carry bit
        self.add_to_register_a(mem);
    }

    pub fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem = self.mem_read(addr);
        self.set_register_a(mem & self.register_a);
    }

    pub fn asl_accumulator(&mut self) {
        let mut mem = self.register_a;

        if mem >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        mem = mem << 1;
        self.set_register_a(mem)
    }
    
    pub fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut mem = self.mem_read(addr);

        if mem >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        mem = mem << 1;
        self.mem_write(addr, mem);
        self.update_zero_and_negative_flags(mem);
    }


    pub fn branch(&mut self, carry_flag: bool) {
        if carry_flag {
            let relative_disp = self.mem_read(self.program_counter) as i8;
            let branch_loc = self.program_counter
                .wrapping_add(1)
                .wrapping_add(relative_disp as u16);
            
            self.program_counter = branch_loc;
        }
    }

    pub fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem = self.mem_read(addr);

        // If the result of the AND is zero, then set zero flag
        // Otherwise, turn it off
        if self.register_a & mem == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        // If 6th bit of memory value is 1, set overflow flag to 1
        // Otherwise set it to 0
        self.status.set(CpuFlags::OVERFLOW, mem & CpuFlags::OVERFLOW.bits > 0);

        // If 7th bit of memory value is 1, set negative flag to 1
        // Otherwise, set it to 0
        self.status.set(CpuFlags::NEGATIVE, mem & CpuFlags::NEGATIVE.bits > 0);
    }

    pub fn clear_carry_flag(&mut self) {
        self.status.remove(CpuFlags::CARRY)
    }

    pub fn clear_decimal_flag(&mut self) {
        self.status.remove(CpuFlags::DECIMAL_MODE)
    }

    pub fn clear_interrupt_flag(&mut self) {
        self.status.remove(CpuFlags::INTERRUPT_DISABLE)
    }

    pub fn clear_overflow_flag(&mut self) {
        self.status.remove(CpuFlags::OVERFLOW)
    }

    pub fn cmp(&mut self, mode: &AddressingMode, register: u8) {
        let addr = self.get_operand_address(mode);
        let mem = self.mem_read(addr);

        // If the register is greater than or equal to mem, set the carry flag
        // If register is equal to mem set the zero flag
        // If bit 7 of the result is set then set negative flag
        if register >= mem {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        // Update zero and negative flags with the result (register value - mem)
        self.update_zero_and_negative_flags(register.wrapping_sub(mem));

    }

    pub fn decrement_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut mem = self.mem_read(addr);
        mem = mem.wrapping_sub(1);
        self.mem_write(addr, mem);
        self.update_zero_and_negative_flags(mem);
    }

    pub fn decrement_register_x(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn decrement_register_y(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn exclusive_or(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem = self.mem_read(addr);
        self.set_register_a(mem ^ self.register_a);
    }

    pub fn increment_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem = self.mem_read(addr);
        let inc_mem = mem.wrapping_add(1);

        self.update_zero_and_negative_flags(inc_mem);
        self.mem_write(addr, inc_mem);
    }

    pub fn increment_register_x(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn increment_register_y(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    // Set program counter to address at the program counter
    pub fn jump_absolute(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter);
        self.program_counter = mem_address;
    }

    // Set the program counter to the address returned at the address of the program counter
    pub fn jump_indirect(&mut self) {
        let addr = self.mem_read_u16(self.program_counter);

        // Does not correctly fetch the target address if it falls on a page boundary.
        // 0x00FF to 0xFFFF.
        // If address is full then this condition is true
        let addr_at_addr = if addr & 0b1111_1111 == 0b1111_1111 {
            let lo = self.mem_read(addr);
            // addr & 0b1111_1111_0000_0000 returns all zeroes
            let hi = self.mem_read(addr & 0xFF00);
            u16::from_le_bytes([lo, hi])
        } else {
            // If not on a page boundary, we can just return the address at the addr fetched
            self.mem_read_u16(addr)
        };

        self.program_counter = addr_at_addr;
    }

    pub fn jump_sub_routine(&mut self) {
        // Push the address - 1 of return point on to the stack and then set the program counter to
        // the target memory address
        // Not sure of the + 2 here. I think it's because we haven't added those ops at this stage
        // so we need to include them here pre-emptively
        self.stack_push_u16(self.program_counter + 2 - 1);
        self.program_counter = self.mem_read_u16(self.program_counter);
    }

    pub fn load_a_register(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);

        // If A is zero, then we need to set the zero flag to 1
        self.set_register_a(value);
    }

    pub fn load_x_register(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_x(value);
    }

    pub fn load_y_register(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_y(value);
    }

    pub fn logical_shift_right_accumulator(&mut self) {
        let mut data = self.register_a;

        if data & CpuFlags::CARRY.bits == CpuFlags::CARRY.bits {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        data = data >> 1;
        self.set_register_a(data);
    }

    pub fn logical_shift_right(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut mem = self.mem_read(addr);

        if mem & CpuFlags::CARRY.bits == CpuFlags::CARRY.bits {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        mem = mem >> 1;
        self.update_zero_and_negative_flags(mem);
        self.mem_write(addr, mem);
    }

    pub fn logical_inclusive_or(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.set_register_a(self.register_a | self.mem_read(addr));
    }

    pub fn push_processor_status(&mut self) {
        let mut processor_flags = self.status.clone();

        // Set break flags
        processor_flags.insert(CpuFlags::BREAK);
        processor_flags.insert(CpuFlags::BREAK2);
        self.stack_push(processor_flags.bits());
    }

    pub fn pull_accumulator(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }

    pub fn pull_processor_status(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);
    }

    pub fn rotate_left_accumulator(&mut self) {
        let mut data = self.register_a;
        let current_carry = self.status.contains(CpuFlags::CARRY);

        // Carry flag is set to the value of bit 7 from unshifted data
        if data >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        // Left shit data by 1
        data = data << 1;

        // Bit zero (carry flag) is filled with the previous bit zero (carry flag) from status 
        if current_carry {
            data = data | CpuFlags::CARRY.bits;
        } else {
            data = data & 0b1111_1110;
        }

        self.set_register_a(data);
    }

    pub fn rotate_left(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut mem = self.mem_read(addr);

        let current_carry = self.status.contains(CpuFlags::CARRY);

        if mem >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        // Left shit mem by 1
        mem = mem << 1;

        // Bit zero (carry flag) is filled with the previous bit zero (carry flag) from status 
        if current_carry {
            mem = mem | CpuFlags::CARRY.bits;
        } else {
            mem = mem & 0b1111_1110;
        }

        self.update_negative_flags(mem);
        self.mem_write(addr, mem);
    }

    pub fn rotate_right_accumulator(&mut self) {
        let mut data = self.register_a;
        let current_carry = self.status.contains(CpuFlags::CARRY);

        // Bit 0 is set to new carry flag value
        if data & CpuFlags::CARRY.bits == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        // right shit data by 1
        data = data >> 1;

        // Bit 7 is filled with current value of carry flag
        if current_carry {
            data = data | CpuFlags::NEGATIVE.bits;
        } else {
            data = data & 0b0111_1111;
        }

        self.set_register_a(data);
    }

    pub fn rotate_right(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut mem = self.mem_read(addr);

        let current_carry = self.status.contains(CpuFlags::CARRY);

        // Bit 0 is set to new carry flag value
        if mem & CpuFlags::CARRY.bits == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        // right shit mem by 1
        mem = mem >> 1;

        // Bit 7 is filled with current value of carry flag
        if current_carry {
            mem = mem | CpuFlags::NEGATIVE.bits;
        } else {
            mem = mem & 0b0111_1111;
        }

        self.update_negative_flags(mem);
        self.mem_write(addr, mem);
    }

    pub fn return_from_interrupt(&mut self) {
        self.status.bits = self.stack_pop();
       
        // Handle break flags
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);

        self.program_counter = self.stack_pop_u16();
    }

    pub fn subtract_with_carry(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(&mode);
        let mem = self.mem_read(addr);

        // A - M - (1 - C)
        // A - M - 1 + C
        // Same as adc method which represents: A + M + C
        self.add_to_register_a(((mem as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    pub fn set_carry_flag(&mut self) {
        self.status.insert(CpuFlags::CARRY);
    }

    pub fn set_decimal_flag(&mut self) {
        self.status.insert(CpuFlags::DECIMAL_MODE);
    }

    pub fn set_interrupt_disable(&mut self) {
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }

    pub fn store_accumulator(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    pub fn store_x_register(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    pub fn store_y_register(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }


    pub fn transfer_accumulator_x(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn transfer_accumulator_y(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn transfer_stack_pointer_to_x(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn transfer_x_accumulator(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn transfer_x_to_stack_pointer(&mut self) {
        self.stack_pointer = self.register_x;
    }

    pub fn transfer_y_accumulator(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }


}
