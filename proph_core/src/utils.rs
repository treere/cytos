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
pub fn rearrange_from_zig_zag(inp: &[i32; 64], outp: &mut [i32; 64]) {
    const INDICES: [usize; 64] = [
        1, 2, 6, 7, 15, 16, 28, 29, //
        3, 5, 8, 14, 17, 27, 30, 43, //
        4, 9, 13, 18, 26, 31, 42, 44, //
        10, 12, 19, 25, 32, 41, 45, 54, //
        11, 20, 24, 33, 40, 46, 53, 55, //
        21, 23, 34, 39, 47, 52, 56, 61, //
        22, 35, 38, 48, 51, 57, 60, 62, //
        36, 37, 49, 50, 58, 59, 63, 64,
    ];

    for i in 0..64 {
        outp[i] = inp[INDICES[i] - 1]
    }
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
