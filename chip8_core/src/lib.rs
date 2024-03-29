use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const START_ADDR: u16 = 0x200;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[0..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & Execute
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                println!("B E E P");
            };
            self.st -= 1;
        }
    }

    pub fn get_display(&mut self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;
        
        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // 0000 No-op 
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH], // 00E0 Clear screen
            (0, 0, 0xE, 0xE) => self.pc = self.pop(), // 00EE Return from subroutine
            (1, _, _, _) => { // 1NNN Jump to address 0xNNN
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }, 
            (2, _, _, _) => { // 2NNN Enter subroutine at 0xNNN
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }, 
            (3, _, _, _) => { // 3XNN skip if VX == 0xNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => { // 4XNN skip if VX != 0xNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => { // 5XY0 skip if VX == VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (9, _, _, 0) => { // 9XY0 skip if VX != VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => { // 6XNN VX = 0xNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            (7, _, _, _) => { // 7XNN VX += 0xNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            (8, _, _, 0) => { // 8XY0 VX = VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            (8, _, _, 1) => { // 8XY1 VX |= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            (8, _, _, 2) => { // 8XY2 VX &= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            (8, _, _, 3) => { // 8XY3 VX ^= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            (8, _, _, 4) => { // 8XY4 VX += VY; sets VF if carry
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 5) => { // 8XY5 VX -= VY; clears VF if borrow
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 }; 

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 6) => { // 8XY6 VX >>= VY; store dropped bit in VF
                let x = digit2 as usize;

                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            (8, _, _, 7) => { // 8XY7 VX = VY - VX; clears VF if borrow
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 0xE) => { // 8XYE VX <<= VY; store dropped bit in VF
                let x = digit2 as usize;

                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = lsb;

            },
            (0xA, _, _, _) => { // ANNN Set index register I to value NNN
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }, 
            (0xB, _, _, _) => { // BNNN Jump to V0 + 0xNNN
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }, 
            (0xC, _, _, _) => { // CXNN  VX = rand() & 0xNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            (0xD, _, _, _) => { // DXYN Draws Sprite
                let x = digit2 as usize;
                let y = digit3 as usize;
                let n = digit4 as u8;
                
                self.v_reg[0xF] = 0;
                let mut sprite_addr = self.i_reg as usize;
                let vx = self.v_reg[x]; // & (SCREEN_WIDTH as u8);
                let vy = self.v_reg[y]; // & (SCREEN_HEIGHT as u8);
                let mut coords = vy as usize * (SCREEN_WIDTH as usize) + vx as usize;
                
                for _row in 0..n {
                    let byte = self.ram[sprite_addr];                
                    for bit in (0..8).rev().map(|n| (byte >> n) & 1) {
                        print!("{bit}");
                        self.screen[coords] = if bit == 1 {true} else {false};
                        coords += 1;
                    }
                    print!("\n");
                    sprite_addr += 1;
                    coords += SCREEN_WIDTH - 8;
                }
            },
            (0xE, _, 9, 0xE) => { // EX9E skip if key vx is pressed
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                if self.keys[vx as usize] {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => { // EXA1 skip if key vx is not pressed
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                if !self.keys[vx as usize] {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => { // FX07 set VX to delay timer
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            (0xF, _, 1, 5) => { // FX15 set delay timer to VX
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            (0xF, _, 1, 8) => { // FX18 set sound timer to VX
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            (0xF, _, 1, 0xE) => { // FX1E I += VX
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            (0xF, _, 0, 0xA) => { // FX0A Get Key
                let x = digit2 as usize;
                let mut key_pressed = false;
                for key in 0..NUM_KEYS {
                    if self.keys[key] {
                        self.v_reg[x] = key as u8;
                        key_pressed = true;
                    }
                }
                if !key_pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 2, 9) => { // FX29 Font Character
                let x = digit2 as usize;
                let character = self.v_reg[x];
                self.i_reg = (character * 5) as u16;
            },
            (0xF, _, 3, 3) => { // FX33 Binary-coded decimal conversion
                let x = digit2 as usize;
                let vx = self.v_reg[x];

                let hundreds = vx / 100;
                let tens = (vx - (hundreds * 100)) / 10;
                let ones = vx - (hundreds * 100) - (tens * 10); 

                let start_addr = self.i_reg as usize;
                match (hundreds, tens, ones) {
                    (0, 0, 0) => self.ram[start_addr] = 0,
                    (0, 0, _) => self.ram[start_addr] = ones,
                    (0, _, _) => {
                        self.ram[start_addr] = tens;
                        self.ram[start_addr + 1] = ones;
                    },
                    (_, _, _) => {
                        self.ram[start_addr] = hundreds;
                        self.ram[start_addr + 1] = tens;
                        self.ram[start_addr + 2] = ones;
                    },
                }
            },
            (0xF, _, 5, 5) => { // FX55 Store memory
                let x = digit2 as usize;
                let mut addr = self.i_reg as usize;
                for register in 0..=x {
                    self.ram[addr] = self.v_reg[register];
                    addr += 1;
                }
            },
            (0xF, _, 6, 5) => { // FX65 Load memory
                let x = digit2 as usize;
                let mut addr = self.i_reg as usize;
                for register in 0..=x {
                    self.v_reg[register] = self.ram[addr];
                    addr += 1;
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:X?}", op),
        }
    } 

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
}