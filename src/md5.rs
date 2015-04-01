use std::num::Int;
use std::{mem, ptr};
use serialize::hex::ToHex;

pub struct Md5 {
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
    buf: Vec<u8>,
    len: u64
}

impl Md5 {
    pub fn new() -> Md5 {
        Md5 {
            s0: 0x67452301,
            s1: 0xefcdab89,
            s2: 0x98badcfe,
            s3: 0x10325476,
            buf: Vec::new(),
            len: 0
        }
    }

    pub fn write(&mut self, input: &[u8]) {
        let mut buf = self.buf.clone();
        self.buf.clear();
        buf.push_all(input);

        for chunk in buf.chunks(16) {
            if chunk.len() == 16 {
                self.process_block(chunk);
                self.len += 16;
            } else {
                self.buf.push_all(chunk);
            }
        }
    }

    pub fn hexdigest(mut self) -> String {
        self.finish().to_hex()
    }

    pub fn finish(mut self) -> Vec<u8> {
        let mut buf = self.buf.clone();
        self.buf.clear();

        let len = self.len + buf.len() as u64;

        buf.push(0x80);
        for _ in 0..(64 - (len + 9) % 64) {
            buf.push(0x00);
        }

        buf.push_all(unsafe { &mem::transmute::<u64, [u8; 8]>((len * 8).to_be()) });

        for chunk in buf.chunks(16) {
            self.process_block(chunk);
        }

        let mut result = Vec::new();
        unsafe {
            result.push_all(&mem::transmute::<u32, [u8; 4]>(self.s0.to_be()));
            result.push_all(&mem::transmute::<u32, [u8; 4]>(self.s1.to_be()));
            result.push_all(&mem::transmute::<u32, [u8; 4]>(self.s2.to_be()));
            result.push_all(&mem::transmute::<u32, [u8; 4]>(self.s3.to_be()));
        }
        result
    }

    fn process_block(&mut self, input: &[u8]) {
        fn f(x: u32, y: u32, z: u32) -> u32 {
            (x & y) | (!x & z)
        }

        fn g(x: u32, y: u32, z: u32) -> u32 {
            (x & z) | (y & !z)
        }

        fn h(x: u32, y: u32, z: u32) -> u32 {
            x ^ y ^ z
        }

        fn j(x: u32, y: u32, z: u32) -> u32 {
            y ^ (x | !z)
        }

        fn op<F: Fn(u32, u32, u32) -> u32>(a: u32, b: u32, c: u32, d: u32, f: F, x: u32, m: u32, s: u32) -> u32 {
            a.wrapping_add(f(b, c, d)).wrapping_add(x).wrapping_add(m).rotate_left(s).wrapping_add(b)
        }

        let mut a = self.s0;
        let mut b = self.s1;
        let mut c = self.s2;
        let mut d = self.s3;

        let mut data = [0u32; 16];

        unsafe {
            let mut x: *mut u32 = data.get_unchecked_mut(0);
            let mut y: *const u8 = input.get_unchecked(0);
            for _ in 0..data.len() {
                let mut tmp: u32 = mem::uninitialized();
                ptr::copy_nonoverlapping(&mut tmp as *mut _ as *mut u8, y, 4);
                *x = Int::from_le(tmp);
                x = x.offset(1);
                y = y.offset(4);
            }
        }

        // round 1
        for i in (0..16).step_by(4) {
            a = op(a, b, c, d, f, data[i], C1[i], 7);
            d = op(d, a, b, c, f, data[i + 1], C1[i + 1], 12);
            c = op(c, d, a, b, f, data[i + 2], C1[i + 2], 17);
            b = op(b, c, d, a, f, data[i + 3], C1[i + 3], 22);
        }

        // round 2
        let mut t = 1;
        for i in (0..16).step_by(4) {
            a = op(a, b, c, d, g, data[t & 0x0f], C2[i], 5);
            d = op(d, a, b, c, g, data[(t + 5) & 0x0f], C2[i + 1], 9);
            c = op(c, d, a, b, g, data[(t + 10) & 0x0f], C2[i + 2], 14);
            b = op(b, c, d, a, g, data[(t + 15) & 0x0f], C2[i + 3], 20);
            t += 20;
        }

        // round 3
        t = 5;
        for i in (0..16).step_by(4) {
            a = op(a, b, c, d, h, data[t & 0x0f], C3[i], 4);
            d = op(d, a, b, c, h, data[(t + 3) & 0x0f], C3[i + 1], 11);
            c = op(c, d, a, b, h, data[(t + 6) & 0x0f], C3[i + 2], 16);
            b = op(b, c, d, a, h, data[(t + 9) & 0x0f], C3[i + 3], 23);
            t += 12;
        }

        // round 4
        t = 0;
        for i in (0..16).step_by(4) {
            a = op(a, b, c, d, j, data[t & 0x0f], C4[i], 6);
            d = op(d, a, b, c, j, data[(t + 7) & 0x0f], C4[i + 1], 10);
            c = op(c, d, a, b, j, data[(t + 14) & 0x0f], C4[i + 2], 15);
            b = op(b, c, d, a, j, data[(t + 21) & 0x0f], C4[i + 3], 21);
            t += 28;
        }

        self.s0 = self.s0.wrapping_add(a);
        self.s1 = self.s1.wrapping_add(b);
        self.s2 = self.s2.wrapping_add(c);
        self.s3 = self.s3.wrapping_add(d);
    }
}

// Round 1 constants
static C1: [u32; 16] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821
];

// Round 2 constants
static C2: [u32; 16] = [
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a
];

// Round 3 constants
static C3: [u32; 16] = [
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665
];

// Round 4 constants
static C4: [u32; 16] = [
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391
];

#[cfg(test)]
mod tests {
    use super::Md5;

    #[test]
    fn test_md5() {
        let mut md5 = Md5::new();
        md5.write(b"Hello world");
        let r = md5.hexdigest();
        let e = "3e25960a79dbc69b674cd4ec67a72c62";
        assert_eq!(r.len(), e.len());
        assert_eq!(&*r, e);
    }
}
