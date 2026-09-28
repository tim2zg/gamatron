use unicorn_engine::{Unicorn, RegisterX86, unicorn_const::{Arch, Mode}};
use unicorn_engine::Prot;

#[derive(Debug)]
pub struct EmulatorState {
    pub registers: [u64; 32], // Assuming x86_64 map roughly to these for basic demo
}

#[derive(Debug)]
pub struct MemoryDelta {
    pub address: u64,
    pub bytes: Vec<u8>,
}

pub trait EmulatorEngine {
    fn load_state(&mut self, state: &EmulatorState) -> Result<(), &'static str>;
    fn apply_memory_deltas(&mut self, deltas: &[MemoryDelta]) -> Result<(), &'static str>;
    fn step_block(&mut self, start_pc: u64, end_pc: u64) -> Result<EmulatorState, &'static str>;
}

pub struct UnicornSandbox<'a> {
    engine: Unicorn<'a, ()>,
}

impl<'a> UnicornSandbox<'a> {
    pub fn new() -> Result<Self, &'static str> {
        let engine = Unicorn::new(Arch::X86, Mode::MODE_64)
            .map_err(|_| "Failed to initialize Unicorn engine")?;
        Ok(Self { engine })
    }
}

impl<'a> EmulatorEngine for UnicornSandbox<'a> {
    fn load_state(&mut self, state: &EmulatorState) -> Result<(), &'static str> {
        // Map 0 to RAX, 1 to RBX as a simple demo
        self.engine.reg_write(RegisterX86::RAX, state.registers[0]).map_err(|_| "Failed to write RAX")?;
        self.engine.reg_write(RegisterX86::RBX, state.registers[1]).map_err(|_| "Failed to write RBX")?;
        Ok(())
    }

    fn apply_memory_deltas(&mut self, deltas: &[MemoryDelta]) -> Result<(), &'static str> {
        for delta in deltas {
            // Align memory to 4KB pages
            let page_addr = delta.address & !0xFFF;
            let size = ((delta.bytes.len() as u64 + 0xFFF) & !0xFFF) as u64;

            // Map the memory if not already mapped (ignoring map errors if already mapped for this simple demo)
            let _ = self.engine.mem_map(page_addr, size, Prot::ALL);

            // Write the actual bytes
            self.engine.mem_write(delta.address, &delta.bytes).map_err(|_| "Failed to write memory")?;
        }
        Ok(())
    }

    fn step_block(&mut self, start_pc: u64, end_pc: u64) -> Result<EmulatorState, &'static str> {
        // Start execution. Time limit = 0 (infinite), instruction limit = 1000 for safety
        self.engine.emu_start(start_pc, end_pc, 0, 1000).map_err(|_| "Emulation failed")?;

        let mut next_state = EmulatorState { registers: [0; 32] };
        next_state.registers[0] = self.engine.reg_read(RegisterX86::RAX).map_err(|_| "Failed to read RAX")?;
        next_state.registers[1] = self.engine.reg_read(RegisterX86::RBX).map_err(|_| "Failed to read RBX")?;
        Ok(next_state)
    }
}
