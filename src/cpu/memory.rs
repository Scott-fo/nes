use super::CPU;

pub trait Memory {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_read_u16(&self, pos: u16) -> u16;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn mem_write_u16(&mut self, pos: u16, data: u16);
}


impl Memory for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        // Using little endian. MSB is stored after the LSB
        // [LSB, MSB]
        let lo = self.mem_read(pos); 
        let hi = self.mem_read(pos + 1);
        u16::from_le_bytes([lo, hi])
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }


    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let [lo, hi] = data.to_le_bytes();

        // Using little endian. MSB is stored after the LSB
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}


