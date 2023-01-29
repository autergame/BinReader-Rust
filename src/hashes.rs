use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub fn fnv1a(string: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for c in string.chars() {
        hash = (hash ^ c.to_ascii_lowercase() as u32).wrapping_mul(0x01000193);
    }
    hash
}

pub fn xxhash(string: &str) -> u64 {
    let str_len = string.len() as u64;
    let mut str_cursor = Cursor::new(string);
    let mut h64: u64;

    if str_len >= 32 {
        let str_limit: u64 = str_len - 32;
        let mut v1: u64 = PRIME1.wrapping_add(PRIME2);
        let mut v2: u64 = PRIME2;
        let mut v3: u64 = 0;
        let mut v4: u64 = PRIME1.wrapping_neg();

        loop {
            v1 = v1.wrapping_add(xxh_read64(&mut str_cursor).wrapping_mul(PRIME2));
            v1 = xxh_rotl64(v1, 31);
            v1 = v1.wrapping_mul(PRIME1);

            v2 = v2.wrapping_add(xxh_read64(&mut str_cursor).wrapping_mul(PRIME2));
            v2 = xxh_rotl64(v2, 31);
            v2 = v2.wrapping_mul(PRIME1);

            v3 = v3.wrapping_add(xxh_read64(&mut str_cursor).wrapping_mul(PRIME2));
            v3 = xxh_rotl64(v3, 31);
            v3 = v3.wrapping_mul(PRIME1);

            v4 = v4.wrapping_add(xxh_read64(&mut str_cursor).wrapping_mul(PRIME2));
            v4 = xxh_rotl64(v4, 31);
            v4 = v4.wrapping_mul(PRIME1);

            if str_cursor.position() <= str_limit {
                break;
            }
        }

        h64 = xxh_rotl64(v1, 1).wrapping_add(
            xxh_rotl64(v2, 7).wrapping_add(xxh_rotl64(v3, 12).wrapping_add(xxh_rotl64(v4, 18))),
        );

        v1 = v1.wrapping_mul(PRIME2);
        v1 = xxh_rotl64(v1, 31);
        v1 = v1.wrapping_mul(PRIME1);
        h64 ^= v1;
        h64 = h64.wrapping_mul(PRIME1).wrapping_add(PRIME4);

        v2 = v2.wrapping_mul(PRIME2);
        v2 = xxh_rotl64(v2, 31);
        v2 = v2.wrapping_mul(PRIME1);
        h64 ^= v2;
        h64 = h64.wrapping_mul(PRIME1).wrapping_add(PRIME4);

        v3 = v3.wrapping_mul(PRIME2);
        v3 = xxh_rotl64(v3, 31);
        v3 = v3.wrapping_mul(PRIME1);
        h64 ^= v3;
        h64 = h64.wrapping_mul(PRIME1).wrapping_add(PRIME4);

        v4 = v4.wrapping_mul(PRIME2);
        v4 = xxh_rotl64(v4, 31);
        v4 = v4.wrapping_mul(PRIME1);
        h64 ^= v4;
        h64 = h64.wrapping_mul(PRIME1).wrapping_add(PRIME4);
    } else {
        h64 = PRIME5;
    }

    h64 = h64.wrapping_add(str_len);

    while str_cursor.position() + 8 <= str_len {
        let mut k1 = xxh_read64(&mut str_cursor);
        k1 = k1.wrapping_mul(PRIME2);
        k1 = xxh_rotl64(k1, 31);
        k1 = k1.wrapping_mul(PRIME1);
        h64 ^= k1;
        h64 = xxh_rotl64(h64, 27)
            .wrapping_mul(PRIME1)
            .wrapping_add(PRIME4);
    }

    if str_cursor.position() + 4 <= str_len {
        h64 ^= (xxh_read32(&mut str_cursor) as u64).wrapping_mul(PRIME1);
        h64 = xxh_rotl64(h64, 23)
            .wrapping_mul(PRIME2)
            .wrapping_add(PRIME3);
    }

    while str_cursor.position() < str_len {
        h64 ^= (xxh_read8(&mut str_cursor) as u64).wrapping_mul(PRIME5);
        h64 = xxh_rotl64(h64, 11).wrapping_mul(PRIME1);
    }

    h64 ^= h64.wrapping_shr(33);
    h64 = h64.wrapping_mul(PRIME2);
    h64 ^= h64.wrapping_shr(29);
    h64 = h64.wrapping_mul(PRIME3);
    h64 ^= h64.wrapping_shr(32);
    h64
}

fn xxh_read8(cursor: &mut Cursor<&str>) -> u8 {
    cursor.read_u8().expect("Could not read u8 XXHash")
}

fn xxh_read32(cursor: &mut Cursor<&str>) -> u32 {
    cursor
        .read_u32::<LittleEndian>()
        .expect("Could not read u32 XXHash")
}

fn xxh_read64(cursor: &mut Cursor<&str>) -> u64 {
    cursor
        .read_u64::<LittleEndian>()
        .expect("Could not read u64 XXHash")
}

fn xxh_rotl64(x: u64, r: u64) -> u64 {
    (x << r) | (x >> (64 - r))
}

const PRIME1: u64 = 0x9E3779B185EBCA87;
const PRIME2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME3: u64 = 0x165667B19E3779F9;
const PRIME4: u64 = 0x85EBCA77C2B2AE63;
const PRIME5: u64 = 0x27D4EB2F165667C5;
