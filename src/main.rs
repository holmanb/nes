use std::num::Wrapping;

type Wu8 = Wrapping<u8>;


pub struct CPU {
    pub register_a: Wu8,
    pub register_x: Wu8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: Wrapping(0),
            register_x: Wrapping(0),
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

   fn mem_read_u16(&mut self, pos: u16) -> u16 {
       let lo = self.mem_read(pos) as u16;
       let hi = self.mem_read(pos + 1) as u16;
       (hi << 8) | (lo as u16)
   }

   fn mem_write_u16(&mut self, pos: u16, data: u16) {
       let hi = (data >> 8) as u8;
       let lo = (data & 0xff) as u8;
       self.mem_write(pos, lo);
       self.mem_write(pos + 1, hi);
   }


   pub fn reset(&mut self) {
       self.register_a = Wrapping(0);
       self.register_x = Wrapping(0);
       self.status = 0;

       self.program_counter = self.mem_read_u16(0xFFFC);
   }

   pub fn load(&mut self, program: Vec<u8>) {
       self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
       self.mem_write_u16(0xFFFC, 0x8000);
   }

   pub fn init(&mut self, program: Vec<u8>) {
       self.load(program);
       self.reset();
    }

   pub fn load_and_run(&mut self, program: Vec<u8>) {
       self.load(program);
       self.reset();
       self.run()
   }

    pub fn run(&mut self) {
    // note: we move  intialization of program_counter from here to load function
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                0xA9 => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.lda(Wrapping(param));
                }
                0xAA => self.tax(),
                0xE8 => self.inx(),

                0x00 => return,
                _ => todo!(),
            }
        }
    }

    fn lda(&mut self, value: Wu8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn inx(&mut self) {
        self.register_x += Wrapping(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn update_zero_and_negative_flags(&mut self, result: Wu8) {
        if result == Wrapping(0) {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & Wrapping(0b1000_0000) != Wrapping(0) {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a.0, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

   #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
       assert_eq!(cpu.register_x, Wrapping(0xc1))
   }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.init(vec![0xaa, 0x00]);
        cpu.register_a = Wrapping(10);
        cpu.run();

        assert_eq!(cpu.register_x, Wrapping(10))
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.init(vec![0xe8, 0xe8, 0x00]);
        cpu.register_x = Wrapping(0xff);
        cpu.run();

        assert_eq!(cpu.register_x, Wrapping(1))
    }
}

fn main() {
    println!("hi");
}
