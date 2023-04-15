use crate::parsing::parser_settings::Parser;

use super::read_bits::BitReaderError;

impl Parser {
    #[inline]
    pub fn skip_n_bytes(&mut self, n: u32) {
        self.ptr += n as usize;
    }
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], BitReaderError> {
        // This will likely fail when demo download was cut off and demo
        // ends early
        if self.ptr + n as usize >= self.bytes.len() {
            return Err(BitReaderError::OutOfBytesError);
        }
        let s = &self.bytes[self.ptr..self.ptr + n as usize];
        self.ptr += n as usize;
        Ok(s)
    }
    #[inline]
    pub fn read_varint(&mut self) -> Result<u32, BitReaderError> {
        let mut result: u32 = 0;
        let mut count: u8 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return Ok(result as u32);
            }
            if self.ptr >= self.bytes.len() {
                return Err(BitReaderError::OutOfBytesError);
            }
            b = self.bytes[self.ptr].try_into().unwrap();
            self.ptr += 1;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Ok(result as u32)
    }
}
