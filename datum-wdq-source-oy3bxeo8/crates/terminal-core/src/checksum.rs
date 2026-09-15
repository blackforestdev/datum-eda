/// Incremental Adler-32 used by RFC 1950 zlib streams.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Adler32 {
    a: u32,
    b: u32,
}

impl Adler32 {
    pub const fn new() -> Self {
        Self { a: 1, b: 0 }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        const MODULUS: u32 = 65_521;
        // 5,552 bytes is the largest safe deferred-modulo block for u32.
        for chunk in bytes.chunks(5_552) {
            for &byte in chunk {
                self.a += u32::from(byte);
                self.b += self.a;
            }
            self.a %= MODULUS;
            self.b %= MODULUS;
        }
    }

    pub const fn finish(self) -> u32 {
        (self.b << 16) | self.a
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn adler32(bytes: &[u8]) -> u32 {
    let mut checksum = Adler32::new();
    checksum.update(bytes);
    checksum.finish()
}

/// Incremental ISO 3309/PNG CRC-32 over the supplied bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Crc32(u32);

impl Crc32 {
    pub const fn new() -> Self {
        Self(0xffff_ffff)
    }

    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u32::from(byte);
            for _ in 0..8 {
                let mask = 0u32.wrapping_sub(self.0 & 1);
                self.0 = (self.0 >> 1) ^ (0xedb8_8320 & mask);
            }
        }
    }

    pub const fn finish(self) -> u32 {
        !self.0
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut checksum = Crc32::new();
    checksum.update(bytes);
    checksum.finish()
}
