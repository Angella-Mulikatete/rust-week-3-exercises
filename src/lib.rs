use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        // TODO: Construct a CompactSize from a u64 value
        Self { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:
        // [0x00–0xFC] => 1 byte
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)
        match self.value {
            0..=0xFC => vec![self.value as u8],
            0xFD..=0xFFFF => {
                let mut buf = vec![0xFD];
                buf.extend_from_slice(&(self.value as u16).to_le_bytes());
                buf
            }
            0x10000..=0xFFFFFFFF => {
                let mut buf = vec![0xFE];
                buf.extend_from_slice(&(self.value as u32).to_le_bytes());
                buf
            }
            _ => {
                let mut buf = vec![0xFF];
                buf.extend_from_slice(&self.value.to_le_bytes());
                buf
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        // Check that enough bytes are available based on prefix.
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }

        let first_byte = bytes[0];
        match first_byte {
            0x00..=0xFC => {
                // Single byte value
                Ok((
                    CompactSize {
                        value: first_byte as u64,
                    },
                    1,
                ))
            }
            0xFD => {
                if bytes.len() < 3 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
                Ok((CompactSize { value }, 3))
            }
            0xFE => {
                if bytes.len() < 5 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                Ok((Self::new(value), 5))
            }
            0xFF => {
                if bytes.len() < 9 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u64::from_le_bytes([
                    bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
                ]);
                Ok((Self::new(value), 9))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        let hex_string = hex::encode(self.0);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32
        let hex_string = String::deserialize(deserializer)?;
        let bytes = hex::decode(hex_string).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom(
                "Invalid Txid length, expected 32 bytes",
            ));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Txid(array))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        Self {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut buf = Vec::with_capacity(36);
        buf.extend_from_slice(&self.txid.0);
        buf.extend_from_slice(&self.vout.to_le_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(&bytes[0..32]);
        let vout = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        Ok((OutPoint::new(txid_bytes, vout), 36))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Self { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let length = CompactSize::new(self.bytes.len() as u64);
        let mut result = length.to_bytes();
        result.extend_from_slice(&self.bytes);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes
        let (length, len_size) = CompactSize::from_bytes(bytes)?;
        let total_len = len_size + length.value as usize;
        if bytes.len() < total_len {
            return Err(BitcoinError::InsufficientBytes);
        }
        let script_bytes = bytes[len_size..total_len].to_vec();
        Ok((Self::new(script_bytes), total_len))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        // TODO: Basic constructor
        Self {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.previous_output.to_bytes());
        buf.extend_from_slice(&self.script_sig.to_bytes());
        buf.extend_from_slice(&self.sequence.to_le_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let mut offset = 0;

        //parse OutPoint
        let (previous_output, size) = OutPoint::from_bytes(&bytes[offset..])?;
        offset += size;

        let (script_sig, size) = Script::from_bytes(&bytes[offset..])?;
        offset += size;

        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let sequence = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        Ok((
            TransactionInput::new(previous_output, script_sig, sequence),
            offset,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        // TODO: Construct a transaction from parts
        Self {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        // - version (4 bytes LE)
        // - CompactSize (number of inputs)
        // - each input serialized
        // - lock_time (4 bytes LE)
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_le_bytes());
        // buf.extend_from_slice(&CompactSize::new(self.inputs.len() as u64).to_bytes());

        // Input count
        let input_count = CompactSize::new(self.inputs.len() as u64);
        buf.extend_from_slice(&input_count.to_bytes());

        for input in &self.inputs {
            buf.extend_from_slice(&input.to_bytes());
        }
        buf.extend_from_slice(&self.lock_time.to_le_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Read version, CompactSize for input count
        // Parse inputs one by one
        // Read final 4 bytes for lock_time
        if bytes.len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut offset = 0;

        let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        offset += 4;

        let (input_count, size) = CompactSize::from_bytes(&bytes[offset..])?;
        offset += size;

        //parse inputs
        let mut inputs = Vec::new();
        for _ in 0..input_count.value {
            let (input, size) = TransactionInput::from_bytes(&bytes[offset..])?;
            inputs.push(input);
            offset += size;
        }

        //parse lock_time
        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let lock_time = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        Ok((BitcoinTransaction::new(version, inputs, lock_time), offset))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly string showing version, inputs, lock_time
        // Display scriptSig length and bytes, and previous output info
        write!(f, "Version: {}", self.version)?;
        writeln!(f, "Inputs({}):", self.inputs.len())?;

        for (i, input) in self.inputs.iter().enumerate() {
            writeln!(f, "    Input {}:", i)?;
            writeln!(
                f,
                "      Previous TXID: {}",
                hex::encode(input.previous_output.txid.0)
            )?;
            writeln!(
                f,
                "      Previous Output Vout: {}",
                input.previous_output.vout
            )?;
            writeln!(
                f,
                "      Script Sig Length: {}",
                input.script_sig.bytes.len()
            )?;
            writeln!(
                f,
                "      Script Sig: {}",
                hex::encode(&input.script_sig.bytes)
            )?;
            writeln!(f, "      Sequence: 0x{:08x}", input.sequence)?;
        }

        writeln!(f, "Lock Time: {}", self.lock_time)?;
        Ok(())
    }
}
