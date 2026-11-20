use serde_json::Value;
use std::io::{self, BufRead, Write};

pub fn read(input: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if line.trim().is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("Content-Length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let size = length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid Content-Length"))?;
    let mut bytes = vec![0; size];
    input.read_exact(&mut bytes)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

pub fn write(output: &mut impl Write, value: Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(&value)?;
    write!(output, "Content-Length: {}\r\n\r\n", bytes.len())?;
    output.write_all(&bytes)?;
    output.flush()
}
