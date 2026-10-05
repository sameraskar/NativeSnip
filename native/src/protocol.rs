use std::io::{self, Read, Write};
pub const LIMIT: usize = 64 * 1024;
pub fn read<R: Read>(r: &mut R) -> io::Result<serde_json::Value> {
    let mut len = [0; 4];
    r.read_exact(&mut len)?;
    let n = u32::from_ne_bytes(len) as usize;
    if n == 0 || n > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid native message length",
        ));
    }
    let mut b = vec![0; n];
    r.read_exact(&mut b)?;
    serde_json::from_slice(&b).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
pub fn write<W: Write>(w: &mut W, v: &serde_json::Value) -> io::Result<()> {
    let b = serde_json::to_vec(v)?;
    if b.len() > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Response too large",
        ));
    }
    w.write_all(&(b.len() as u32).to_ne_bytes())?;
    w.write_all(&b)?;
    w.flush()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let v = serde_json::json!({"op":"ping"});
        let mut b = vec![];
        write(&mut b, &v).unwrap();
        assert_eq!(read(&mut &b[..]).unwrap(), v);
    }
    #[test]
    fn limits_and_truncation() {
        for b in [0u32.to_ne_bytes(), (LIMIT as u32 + 1).to_ne_bytes()] {
            assert!(read(&mut &b[..]).is_err());
        }
        assert!(read(&mut &[1u8, 0][..]).is_err());
    }
    #[test]
    fn bad_json() {
        let mut b = 1u32.to_ne_bytes().to_vec();
        b.push(b'!');
        assert!(read(&mut &b[..]).is_err());
    }
}
