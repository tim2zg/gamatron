use duckdb::{Connection, Result};

pub struct Storage {
    conn: Connection,
}

#[derive(Debug)]
pub struct ThreadRecord {
    pub thread_id: u32,
    pub created_timestamp: u64,
    pub start_pc: u64,
}

#[derive(Debug)]
pub struct BlockTraceRecord {
    pub trace_id: u64,
    pub thread_id: u32,
    pub block_address: u64,
    pub instruction_count: u16,
    pub timestamp: u64,
    pub parent_group_id: Option<u64>,
}

#[derive(Debug)]
pub struct RegisterDiffRecord {
    pub trace_id: u64,
    pub reg_index: u8,
    pub value_before: u64,
    pub value_after: u64,
}

#[derive(Debug)]
pub struct MemoryDeltaRecord {
    pub delta_id: u64,
    pub trace_id: u64,
    pub address: u64,
    pub bytes_before: Vec<u8>,
    pub bytes_after: Vec<u8>,
    pub is_stack: bool,
    pub is_heap: bool,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Storage { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            -- Thread Contexts
            CREATE TABLE IF NOT EXISTS threads (
                thread_id UINTEGER PRIMARY KEY,
                created_timestamp UBIGINT,
                start_pc UBIGINT
            );

            -- Basic Block Trace Timeline
            CREATE TABLE IF NOT EXISTS block_trace (
                trace_id UBIGINT PRIMARY KEY,
                thread_id UINTEGER,
                block_address UBIGINT,
                instruction_count USMALLINT,
                timestamp UBIGINT,
                parent_group_id UBIGINT
            );

            -- Register State Diffs per Block Execution
            CREATE TABLE IF NOT EXISTS register_diffs (
                trace_id UBIGINT REFERENCES block_trace(trace_id),
                reg_index UTINYINT,
                value_before UBIGINT,
                value_after UBIGINT
            );

            -- Memory Mutation Logs (Page Deltas)
            CREATE TABLE IF NOT EXISTS memory_deltas (
                delta_id UBIGINT PRIMARY KEY,
                trace_id UBIGINT REFERENCES block_trace(trace_id),
                address UBIGINT,
                bytes_before BLOB,
                bytes_after BLOB,
                is_stack BOOLEAN,
                is_heap BOOLEAN
            );

            -- Hierarchical Groups & User/AI Annotations
            CREATE TABLE IF NOT EXISTS semantic_groups (
                group_id UBIGINT PRIMARY KEY,
                parent_group_id UBIGINT,
                name VARCHAR,
                description TEXT,
                start_trace_id UBIGINT,
                end_trace_id UBIGINT,
                created_by VARCHAR -- 'USER' or 'MCP_LLM'
            );
            ",
        )?;
        Ok(())
    }

    pub fn insert_thread(&self, record: ThreadRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO threads (thread_id, created_timestamp, start_pc) VALUES (?, ?, ?)",
            [record.thread_id, record.created_timestamp as u32, record.start_pc as u32],
        )?;
        Ok(())
    }

    pub fn insert_block_trace(&self, record: BlockTraceRecord) -> Result<()> {
        let parent_group_id = record.parent_group_id.unwrap_or(0); // Simple handling for Option
        self.conn.execute(
            "INSERT INTO block_trace (trace_id, thread_id, block_address, instruction_count, timestamp, parent_group_id) VALUES (?, ?, ?, ?, ?, ?)",
            [
                record.trace_id as u32,
                record.thread_id,
                record.block_address as u32,
                record.instruction_count as u32,
                record.timestamp as u32,
                parent_group_id as u32
            ],
        )?;
        Ok(())
    }

    pub fn insert_register_diff(&self, record: RegisterDiffRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO register_diffs (trace_id, reg_index, value_before, value_after) VALUES (?, ?, ?, ?)",
            [record.trace_id as u32, record.reg_index as u32, record.value_before as u32, record.value_after as u32],
        )?;
        Ok(())
    }

    pub fn insert_memory_delta(&self, record: MemoryDeltaRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO memory_deltas (delta_id, trace_id, address, bytes_before, bytes_after, is_stack, is_heap) VALUES (?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                record.delta_id as u32,
                record.trace_id as u32,
                record.address as u32,
                record.bytes_before,
                record.bytes_after,
                record.is_stack,
                record.is_heap
            ],
        )?;
        Ok(())
    }
}
