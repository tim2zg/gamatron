use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::io;
use storage::Storage;

#[derive(Deserialize, Debug)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

#[derive(Serialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Value,
}

async fn handle_request(req: RpcRequest, storage: &Storage) -> RpcResponse {
    let result = match req.method.as_str() {
        "get_block_diff" => {
            let trace_id = req.params.and_then(|p| p.get("trace_id").and_then(|id| id.as_u64()));
            if let Some(id) = trace_id {
                // Here we would use `storage` to query the diff from DuckDB.
                // Since this is an MVP, we return a mock response that acknowledges the ID.
                json!({ "status": "success", "trace_id": id, "data": format!("diff_data_placeholder_for_{}", id) })
            } else {
                json!({ "error": "Invalid or missing trace_id parameter" })
            }
        }
        "create_semantic_group" => {
            // Placeholder: Create semantic group in `storage`
            json!({ "status": "success", "message": "Group created" })
        }
        _ => json!({ "error": "Method not found" }),
    };

    RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: req.id,
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    // Create a temporary in-memory duckdb for the MCP server instance
    let storage = Storage::new("").unwrap();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        if let Ok(req) = serde_json::from_str::<RpcRequest>(&line) {
            let res = handle_request(req, &storage).await;
            if let Ok(res_str) = serde_json::to_string(&res) {
                stdout.write_all(res_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }
    }

    Ok(())
}
