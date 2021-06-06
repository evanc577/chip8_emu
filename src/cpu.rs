// Display
pub const DISPLAY_W: usize = 64;
pub const DISPLAY_H: usize = 32;

// CPU field sizes
const MEM_SIZE: usize = 0x1000;
const GFX_SIZE: usize = DISPLAY_W * DISPLAY_H;
const REG_V_SIZE: usize = 0x10;
const STACK_SIZE: usize = 0x10;
pub const KEY_SIZE: usize = 0x10;

const PROGRAM_OFFSET: usize = 0x200; // Program load address

// Fontset
const FONTSET_OFFSET: usize = 0x50;
const FONTSET_SIZE: usize = 0x50;
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Clone, Copy, PartialEq)]
pub enum CPUState {
    Running,
    RunningDraw,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Pressed,
    NotPressed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelState {
    On,
    Off,
}

#[derive(Clone, Copy)]
pub struct CycleInput {
    pub keys: [KeyState; KEY_SIZE],
    pub decrement_timer: bool,
}

#[derive(Clone, Copy)]
pub struct CycleOutput<'a> {
    pub state: CPUState,
    pub gfx: &'a [PixelState; GFX_SIZE],
    pub beep: bool,
}

#[allow(non_snake_case)]
pub struct CPU {
    mem: [u8; MEM_SIZE],         //Main memory
    gfx: [PixelState; GFX_SIZE], // Framebuffer

    // general registers
    V: [u8; REG_V_SIZE], // Data registers
    PC: usize,           // Program counter
    prev_PC: usize,
    I: usize, // Address register

    // timers
    delay_timer: u8,
    sound_timer: u8,

    // stack
    stack: [usize; STACK_SIZE],
    SP: usize, // Stack pointer

    // input
    // key: [KeyState; KEY_SIZE],

    // state
    state: CPUState,
}

impl CPU {
    pub fn new(program: &[u8]) -> Self {
        let mut cpu = Self {
            mem: [0; MEM_SIZE],
            gfx: [PixelState::Off; GFX_SIZE],
            V: [0; REG_V_SIZE],
            PC: PROGRAM_OFFSET,
            prev_PC: PROGRAM_OFFSET,
            I: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            SP: 0,
            state: CPUState::Running,
        };

        // copy fontset to main memory
        cpu.mem[FONTSET_OFFSET..FONTSET_OFFSET + FONTSET_SIZE].copy_from_slice(&FONTSET);

        // copy program to main memory
        cpu.mem[PROGRAM_OFFSET..PROGRAM_OFFSET + program.len()].copy_from_slice(program);

        cpu
    }

    pub fn cycle(&mut self, input: &CycleInput) -> CycleOutput {
        self.state = CPUState::Running;
        self.prev_PC = self.PC;

        // Fetch
        let instruction = u16::from(self.mem[self.PC]) << 8 | u16::from(self.mem[self.PC + 1]);
        self.PC += 2;
        // eprintln!("FETCH 0x{:04x}", instruction);

        // Decode
        let opcode = instruction >> 12;

        // Execute
        match opcode {
            0x0 => self.opcode_0(instruction),
            0x1 => self.opcode_1(instruction),
            0x2 => self.opcode_2(instruction),
            0x3 => self.opcode_3(instruction),
            0x4 => self.opcode_4(instruction),
            0x5 => self.opcode_5(instruction),
            0x6 => self.opcode_6(instruction),
            0x7 => self.opcode_7(instruction),
            0x8 => self.opcode_8(instruction),
            0x9 => self.opcode_9(instruction),
            0xA => self.opcode_a(instruction),
            0xB => self.opcode_b(instruction),
            0xC => self.opcode_c(instruction),
            0xD => self.opcode_d(instruction),
            0xE => self.opcode_e(instruction, &input.keys),
            0xF => self.opcode_f(instruction, &input.keys),
            _ => panic!("Unknown instruction 0x{:04x}", instruction),
        };

        // Update timers
        if input.decrement_timer {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }

        CycleOutput {
            state: self.state,
            gfx: &self.gfx,
            beep: self.sound_timer != 0,
        }
    }

    fn opcode_0(&mut self, instruction: u16) {
        match instruction {
            // Clear screen
            0x00E0 => {
                self.gfx.iter_mut().for_each(|b| *b = PixelState::Off);
                self.state = CPUState::RunningDraw;
            }

            // Return from subroutine
            0x00EE => {
                self.PC = self.stack[self.SP];
                self.SP -= 1;
            }

            _ => panic!("Unknown instruction 0x{:04x}", instruction),
        }
    }

    /// 1NNN
    /// Jump to address NNN
    fn opcode_1(&mut self, instruction: u16) {
        self.PC = usize::from(instruction & 0x0FFF);
    }

    /// 2NNN
    /// Call subroutine at address NNN
    fn opcode_2(&mut self, instruction: u16) {
        self.SP += 1;
        self.stack[self.SP] = self.PC;
        self.PC = usize::from(instruction & 0x0FFF);
    }

    /// 3XNN
    /// Skip next instruction if VX == NN
    fn opcode_3(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let nn = instruction as u8;
        if self.V[x] == nn {
            self.PC += 2;
        }
    }

    /// 4XNN
    /// Skip next instruction if VX != NN
    fn opcode_4(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let nn = instruction as u8;
        if self.V[x] != nn {
            self.PC += 2;
        }
    }

    /// 5XY0
    /// Skip next instruction if VX == VY
    fn opcode_5(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let y: usize = get_Y(instruction);
        if self.V[x] == self.V[y] {
            self.PC += 2;
        }
    }

    /// 6XNN
    /// set VX to NN
    fn opcode_6(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let nn = instruction as u8;
        self.V[x] = nn;
    }

    /// 7XNN
    /// Add NN to VX (carry flag not changed)
    fn opcode_7(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let nn = instruction as u8;
        self.V[x] = self.V[x].wrapping_add(nn);
    }

    /// 8XY-
    /// Various arithmetic and bitwise operations
    fn opcode_8(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let y: usize = get_Y(instruction);
        match instruction & 0xF {
            // basic bitwise operations
            0x0 => self.V[x] = self.V[y],
            0x1 => self.V[x] |= self.V[y],
            0x2 => self.V[x] &= self.V[y],
            0x3 => self.V[x] ^= self.V[y],

            // add VY to VX, VF set to 1 if carry, otherwise set to 0
            0x4 => {
                let temp_vx = self.V[x];
                self.V[x] = self.V[x].wrapping_add(self.V[y]);
                if self.V[x] < temp_vx {
                    // carry
                    self.V[0xF] = 1;
                } else {
                    self.V[0xF] = 0;
                }
            }

            // subtract VY from VX, VF set to 1 if borrow, otherwise set to 0
            0x5 => {
                let temp_vx = self.V[x];
                self.V[x] = self.V[x].wrapping_sub(self.V[y]);
                if self.V[x] > temp_vx {
                    // borrow
                    self.V[0xF] = 0;
                } else {
                    self.V[0xF] = 1;
                }
            }

            // (undocumented) stores LSB of VX in VF, then right shifts VX by 1
            0x6 => {
                self.V[0xF] = self.V[x] & 0x1;
                self.V[x] >>= 1;
            }

            // (undocumented) sets VX to (VY - VX), VF set to 1 if borrow, otherwise set to 0
            0x7 => {
                self.V[x] = self.V[y].wrapping_sub(self.V[x]);
                if self.V[x] > self.V[y] {
                    // borrow
                    self.V[0xF] = 0;
                } else {
                    self.V[0xF] = 1;
                }
            }

            // (undocumented) stores MSB of VX in VF, then left shifts VX by 1
            0xE => {
                self.V[0xF] = (self.V[x] & 0x80) >> 7;
                self.V[x] <<= 1;
            }

            _ => panic!("Unknown instruction 0x{:04x}", instruction),
        }
    }

    /// 9XY0
    /// Skip next instruction if VX != VY
    fn opcode_9(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let y: usize = get_Y(instruction);
        if self.V[x] != self.V[y] {
            self.PC += 2;
        }
    }

    /// ANNN
    /// Sets I to the address NNN
    fn opcode_a(&mut self, instruction: u16) {
        self.I = usize::from(instruction & 0x0FFF);
    }

    /// BNNN
    /// Jump to address NNN + V0
    fn opcode_b(&mut self, instruction: u16) {
        self.PC = usize::from((instruction & 0x0FFF).wrapping_add(u16::from(self.V[0])));
    }

    /// CXNN
    /// Sets VX to (rand & NN)
    fn opcode_c(&mut self, instruction: u16) {
        let x: usize = get_X(instruction);
        let nn = instruction as u8;
        let rand_val: u8 = rand::random();
        self.V[x] = rand_val & nn;
    }

    /// DXYN
    /// Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and
    /// a height of N+1 pixels. Each row of 8 pixels is read as bit-coded
    /// starting from memory location I; I value does not change after the
    /// execution of this instruction. VF is set to 1 if any screen pixels are
    /// flipped from set to unset when the sprite is drawn, and to 0 if that
    /// does not happen
    fn opcode_d(&mut self, instruction: u16) {
        let x = get_X::<usize>(instruction);
        let y = get_Y::<usize>(instruction);
        let n = (instruction & 0xF) as usize;
        let vx = usize::from(self.V[x]) % DISPLAY_W;
        let vy = usize::from(self.V[y]) % DISPLAY_H;

        self.V[0xF] = 0;

        let row_end = std::cmp::min(n, DISPLAY_H - vy);
        for row in 0..row_end {
            let sprite_data = self.mem[self.I + row];
            let col_end = std::cmp::min(8, DISPLAY_W - vx);
            for col in 0..col_end {
                let sprite_on = (1 << (7 - col)) & sprite_data != 0;
                let gfx_index = (vy + row) * DISPLAY_W + (vx + col);
                let pix_on = self.gfx[gfx_index] == PixelState::On;
                if sprite_on && pix_on {
                    self.V[0xF] = 0x1;
                }
                self.gfx[gfx_index] = match sprite_on ^ pix_on {
                    true => PixelState::On,
                    false => PixelState::Off,
                }
            }
        }

        self.state = CPUState::RunningDraw;
    }

    /// EX--
    /// Input processing
    fn opcode_e(&mut self, instruction: u16, keys: &[KeyState; KEY_SIZE]) {
        let x: usize = get_X(instruction);
        match instruction & 0x00FF {
            // Skips next instruction if the key stored in VX is pressed
            0x9E => {
                if keys[usize::from(self.V[x])] == KeyState::Pressed {
                    self.PC += 2;
                }
            }

            // Skips next instruction if the key stored in VX is not pressed
            0xA1 => {
                if keys[usize::from(self.V[x])] == KeyState::NotPressed {
                    self.PC += 2;
                }
            }

            _ => panic!("Unknown instruction 0x{:04x}", instruction),
        }
    }

    /// FX--
    /// Misc
    fn opcode_f(&mut self, instruction: u16, keys: &[KeyState; KEY_SIZE]) {
        let x: usize = get_X(instruction);
        match instruction & 0x00FF {
            // Set VX to the value of the delay timer
            0x07 => self.V[x] = self.delay_timer,

            // A key press is awaited, and then stored in VX.
            // (Blocking Operation. All instruction halted until next key event)
            0x0A => {
                if let Some(offset) = keys
                    .iter()
                    .enumerate()
                    .find_map(|(offset, key)| match *key {
                        KeyState::Pressed => Some(offset),
                        KeyState::NotPressed => None,
                    })
                {
                    self.V[x] = offset as u8;
                } else {
                    self.PC -= 2;
                }
            }

            // Set the delay timer to VX
            0x15 => self.delay_timer = self.V[x],

            // Set the sound timer to VX
            0x18 => self.sound_timer = self.V[x],

            // Add VX to I
            0x1E => self.I += usize::from(self.V[x]),

            // Sets I to the location of the sprite for the character in VX.
            // Characters 0-F (in hexadecimal) are represented by a 4x5 font.
            0x29 => self.I = 5 * usize::from(self.V[x] & 0xF) + FONTSET_OFFSET,

            // Stores the binary-coded decimal representation of VX, with the
            // most significant of three digits at the address in I, the middle
            // digit at I plus 1, and the least significant digit at I plus 2.
            0x33 => {
                self.mem[self.I] = self.V[x] / 100;
                self.mem[self.I + 1] = self.V[x] / 10 % 10;
                self.mem[self.I + 2] = self.V[x] % 10;
            }

            // Stores V0 to VX (including VX) in memory starting at address I
            0x55 => {
                for (offset, reg_val) in self.V[0..=usize::from(x)].iter().enumerate() {
                    self.mem[self.I + offset] = *reg_val;
                }
            }

            // Fills V0 to VX (including VX) with values from memory starting at
            // address I
            0x65 => {
                for (offset, mem_val) in self.mem[self.I..=(self.I + usize::from(x))]
                    .iter()
                    .enumerate()
                {
                    self.V[offset] = *mem_val;
                }
            }

            _ => panic!("Unknown instruction 0x{:04x}", instruction),
        }
    }
}

#[allow(non_snake_case)]
fn get_X<T: From<u16>>(instruction: u16) -> T {
    T::from((instruction & 0x0F00) >> 8)
}

#[allow(non_snake_case)]
fn get_Y<T: From<u16>>(instruction: u16) -> T {
    T::from((instruction & 0x00F0) >> 4)
}
