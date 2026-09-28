use emulator::{EmulatorEngine, UnicornSandbox};
use storage::Storage;
use tracer::FridaAgent;

pub struct CoreEngine<'a> {
    pub storage: Storage,
    pub tracer: FridaAgent,
    pub emulator: UnicornSandbox<'a>,
}

impl<'a> CoreEngine<'a> {
    pub fn new(db_path: &str) -> Result<Self, &'static str> {
        let storage = Storage::new(db_path).map_err(|_| "Failed to initialize storage")?;
        let tracer = FridaAgent::new();
        let emulator = UnicornSandbox::new()?;

        Ok(Self {
            storage,
            tracer,
            emulator,
        })
    }
}
