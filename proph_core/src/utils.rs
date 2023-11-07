// /// Rearrange input data which were zigzag into a normal matrix
// ///
// ///  1,  2,  6,  7, 15, 16, 28, 29,
// ///  3,  5,  8, 14, 17, 27, 30, 43,
// ///  4,  9, 13, 18, 26, 31, 42, 44,
// /// 10, 12, 19, 25, 32, 41, 45, 54,
// /// 11, 20, 24, 33, 40, 46, 53, 55,
// /// 21, 23, 34, 39, 47, 52, 56, 61,
// /// 22, 35, 38, 48, 51, 57, 60, 62,
// /// 36, 37, 49, 50, 58, 59, 63, 64,
// pub fn rearrange_from_zig_zag(inp: &[i32; 64], outp: &mut [i32; 64]) {
//     const INDICES: [usize; 64] = [
//         0, 1, 5, 6, 14, 15, 27, 28, //
//         2, 4, 7, 13, 16, 26, 29, 42, //
//         3, 8, 12, 17, 25, 30, 41, 43, //
//         9, 11, 18, 24, 31, 40, 44, 53, //
//         10, 19, 23, 32, 39, 45, 52, 54, //
//         20, 22, 33, 38, 46, 51, 55, 60, //
//         21, 34, 37, 47, 50, 56, 59, 61, //
//         35, 36, 48, 49, 57, 58, 62, 63,
//     ];

//     for i in 0..64 {
//         outp[i] = inp[INDICES[i]]
//     }
// }

pub fn rearrange_from_zig_zag(inp: &[i32; 64], outp: &mut [i32; 64]) {
    outp[0] = inp[0];
    outp[1] = inp[1];
    outp[2] = inp[5];
    outp[3] = inp[6];
    outp[4] = inp[14];
    outp[5] = inp[15];
    outp[6] = inp[27];
    outp[7] = inp[28];
    outp[8] = inp[2];
    outp[9] = inp[4];
    outp[10] = inp[7];
    outp[11] = inp[13];
    outp[12] = inp[16];
    outp[13] = inp[26];
    outp[14] = inp[29];
    outp[15] = inp[42];
    outp[16] = inp[3];
    outp[17] = inp[8];
    outp[18] = inp[12];
    outp[19] = inp[17];
    outp[20] = inp[25];
    outp[21] = inp[30];
    outp[22] = inp[41];
    outp[23] = inp[43];
    outp[24] = inp[9];
    outp[25] = inp[11];
    outp[26] = inp[18];
    outp[27] = inp[24];
    outp[28] = inp[31];
    outp[29] = inp[40];
    outp[30] = inp[44];
    outp[31] = inp[53];
    outp[32] = inp[10];
    outp[33] = inp[19];
    outp[34] = inp[23];
    outp[35] = inp[32];
    outp[36] = inp[39];
    outp[37] = inp[45];
    outp[38] = inp[52];
    outp[39] = inp[54];
    outp[40] = inp[20];
    outp[41] = inp[22];
    outp[42] = inp[33];
    outp[43] = inp[38];
    outp[44] = inp[46];
    outp[45] = inp[51];
    outp[46] = inp[55];
    outp[47] = inp[60];
    outp[48] = inp[21];
    outp[49] = inp[34];
    outp[50] = inp[37];
    outp[51] = inp[47];
    outp[52] = inp[50];
    outp[53] = inp[56];
    outp[54] = inp[59];
    outp[55] = inp[61];
    outp[56] = inp[35];
    outp[57] = inp[36];
    outp[58] = inp[48];
    outp[59] = inp[49];
    outp[60] = inp[57];
    outp[61] = inp[58];
    outp[62] = inp[62];
    outp[63] = inp[63];
}

pub struct RemoveFF00<'a> {
    f: &'a [u8],
    index: usize,
}

impl<'a> RemoveFF00<'a> {
    pub fn new(f: &'a [u8]) -> Self {
        RemoveFF00 { f, index: 0 }
    }

    pub fn len(&self) -> usize {
        self.index
    }
}

impl Iterator for RemoveFF00<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.f[self.index], self.f[self.index + 1]) {
            (0xff, 0x00) => {
                self.index += 2;
                Some(0xff)
            }
            (0xff, _) => None,
            (rest, _) => {
                self.index += 1;
                Some(rest)
            }
        }
    }
}

#[cfg(test)]
mod tests_remove_ff00 {
    use super::*;

    #[test]
    fn count() {
        let v = [0xa0, 0xff, 0x00, 0xa3, 0xff, 0xf3];
        let v = RemoveFF00::new(&v);
        let count = v.count();
        assert_eq!(count, 3);
    }

    #[test]
    fn len() {
        let v = [0xa0, 0xff, 0x00, 0xa3, 0xff, 0xf3];
        let mut v = RemoveFF00::new(&v);

        while let Some(_) = v.next() {}

        assert_eq!(v.len(), 4);
    }

    #[test]
    fn data() {
        let v = [0xa0, 0xff, 0x00, 0xa3, 0xff, 0xf3];

        let v = Vec::from_iter(RemoveFF00::new(&v));

        assert_eq!(vec![0xa0, 0xff, 0xa3], v);
    }
}

pub struct BitIterator<T> {
    current: Option<u8>,
    encoded: T,
    i: u8,
}

impl<T> BitIterator<T> {
    pub fn new(encoded: T) -> Self {
        Self {
            encoded,
            i: 7,
            current: None,
        }
    }
}

impl<T: Iterator<Item = u8>> Iterator for BitIterator<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_none() {
            self.current = self.encoded.next();
        }

        if let Some(byte) = self.current {
            let b = (1u8 << self.i & byte) >> self.i;

            if self.i == 0 {
                self.current = None;
                self.i = 7;
            } else {
                self.i -= 1;
            }
            Some(b)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test_bit_iterator {
    use super::*;

    #[test]
    fn test_using_vec_iter() {
        let v = vec![0b0000_1111];

        assert_eq!(
            BitIterator::new(v.into_iter()).collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 1, 1, 1, 1]
        );
    }

    #[test]
    fn test_using_vec_iter2() {
        let v = vec![0b0000_1111, 0b1111_0000];

        assert_eq!(
            BitIterator::new(v.into_iter()).collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_using_iter() {
        let v = vec![0b0000_1111];
        let mut v = v.into_iter();

        assert_eq!(
            BitIterator::new(&mut v).collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 1, 1, 1, 1]
        );

        assert_eq!(None, v.next());
    }
}

#[cfg(test)]
mod bench_zigzag {
    use super::*;
    use test::{black_box, Bencher};

    #[bench]
    fn zigzag_1(b: &mut Bencher) {
        let inp = [0; 64];
        let mut outp = [0; 64];

        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(rearrange_from_zig_zag(&inp, &mut outp));
            }
        });
    }
}
