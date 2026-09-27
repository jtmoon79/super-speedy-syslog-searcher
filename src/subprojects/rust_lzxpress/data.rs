// Ripped from https://github.com/MagnetForensics/rust-lzxpress/blob/main/src/data.rs (v0.7.1, MIT)
// Modified by @jtmoon79: `compress()` and `store32le!` removed; `Error` path changed;
// a set flag bit with exhausted input ends decompression successfully per
// [MS-XCA] 2.4.4 "Plain LZ77 Decompression Algorithm Details".

use std::mem;

pub use super::error::Error;

macro_rules! load16le {
    ($dst:expr,$src:expr,$idx:expr) => {{
        $dst = (u32::from($src[$idx + 1]) << 8 | u32::from($src[$idx])) as usize;
    }};
}

macro_rules! load32le {
    ($dst:expr,$src:expr,$idx:expr) => {{
        $dst = ((u32::from($src[$idx + 3]) << 24)
            | (u32::from($src[$idx + 2]) << 16)
            | (u32::from($src[$idx + 1]) << 8)
            | u32::from($src[$idx])) as usize;
    }};
}

pub fn decompress(in_buf: &[u8]) -> Result<Vec<u8>, Error> {
    let mut out_idx: usize = 0;
    let mut in_idx: usize = 0;
    let mut nibble_idx: usize = 0;

    let mut flags: usize = 0;
    let mut flag_count: usize = 0;

    let mut length: usize;
    let mut offset: usize;

    let mut out_buf: Vec<u8> = Vec::new();

    while in_idx < in_buf.len() {
        if flag_count == 0 {
            if (in_idx + 3) >= in_buf.len() {
                return Err(Error::MemLimit);
            }

            load32le!(flags, in_buf, in_idx);
            in_idx += mem::size_of::<u32>();
            flag_count = 32;
        }

        flag_count -= 1;

        // Check whether the bit specified by flag_count is set or not
        // set in flags. For example, if flag_count has value 4
        // check whether the 4th bit of the value in flags is set.
        if (flags & (1 << flag_count)) == 0 {
            if in_idx >= in_buf.len() {
                return Err(Error::MemLimit);
            }
            out_buf.push(in_buf[in_idx]);

            in_idx += mem::size_of::<u8>();
            out_idx += mem::size_of::<u8>();
        } else {
            // [MS-XCA] 2.4.4: "If InputPosition == InputLength, decompression is complete"
            if in_idx >= in_buf.len() {
                break;
            }
            if (in_idx + 1) >= in_buf.len() {
                return Err(Error::MemLimit);
            }

            load16le!(length, in_buf, in_idx);
            in_idx += mem::size_of::<u16>();

            offset = (length / 8) + 1;
            length = length % 8;

            if length == 7 {
                if nibble_idx == 0 {
                    if in_idx >= in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = (in_buf[in_idx] % 16).into();
                    nibble_idx = in_idx;
                    in_idx += mem::size_of::<u8>();
                } else {
                    if nibble_idx >= in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = (in_buf[nibble_idx] / 16).into();
                    nibble_idx = 0;
                }

                if length == 15 {
                    if in_idx >= in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = in_buf[in_idx].into();
                    in_idx += mem::size_of::<u8>();

                    if length == 255 {
                        if (in_idx + 1) >= in_buf.len() {
                            return Err(Error::MemLimit);
                        }

                        load16le!(length, in_buf, in_idx);
                        in_idx += mem::size_of::<u16>();

                        if length == 0 {
                            load32le!(length, in_buf, in_idx);
                            in_idx += mem::size_of::<u32>();
                        }

                        if length < 15 + 7 {
                            return Err(Error::CorruptedData);
                        }
                        length -= 15 + 7;
                    }
                    length += 15;
                }
                length += 7;
            }
            length += 3;

            for _i in 0..length {
                if offset > out_idx {
                    return Err(Error::CorruptedData);
                }

                out_buf.push(out_buf[out_idx - offset]);
                out_idx += mem::size_of::<u8>();
            }
        }
    }

    Ok(out_buf)
}
