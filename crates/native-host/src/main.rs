// SPDX-License-Identifier: GPL-3.0-or-later
//! Native Messaging host entry point.

use freeloader_protocol::{validate_request, Request, MAX_PAYLOAD_BYTES};
use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut header = [0_u8; 4];
        match input.read_exact(&mut header) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(error.into()),
        }
        let size = u32::from_le_bytes(header) as usize;
        if size > MAX_PAYLOAD_BYTES {
            return Err("native message exceeds limit".into());
        }
        let mut payload = vec![0_u8; size];
        input.read_exact(&mut payload)?;
        let request: Request = serde_json::from_slice(&payload)?;
        validate_request(&request, payload.len()).map_err(|_| "invalid native message")?;
        let response = serde_json::json!({"version": 1, "type": "ack"});
        let encoded = serde_json::to_vec(&response)?;
        output.write_all(&(encoded.len() as u32).to_le_bytes())?;
        output.write_all(&encoded)?;
        output.flush()?;
    }
    Ok(())
}
