#[derive(Debug, PartialEq)]
pub enum Node {
    Value(u8),
    Split(u32),
}

#[derive(Default, Debug)]
pub struct HuffmanTree {
    tree: Vec<Node>,
}

impl HuffmanTree {
    pub fn compose(&mut self, counting: &[u8], symbol: &[u8]) {
        let last_index = counting
            .iter()
            .rposition(|val| *val != 0)
            .unwrap_or(counting.len() - 1)
            + 1;

        self.tree.clear();
        self.tree.push(Node::Split(1));

        let mut symbol = symbol.iter().cloned().map(Node::Value);

        counting[..last_index - 1].iter().cloned().enumerate().fold(
            (0, 3),
            |(used, link), (index, count)| {
                self.tree.extend((0..count).map(|_| symbol.next().unwrap()));

                let used = 2 * used + count as u32;
                let links = 2u32.pow(index as u32 + 1) - used;

                self.tree
                    .extend((0..links).map(|x| Node::Split(x * 2 + link)));

                (used, link + links * 2)
            },
        );

        (0..counting[last_index - 1]).for_each(|_| self.tree.push(symbol.next().unwrap()));
    }

    pub fn decode<'a>(
        &'a self,
        encoded: impl Iterator<Item = u8> + 'a,
    ) -> impl Iterator<Item = u8> + 'a {
        HuffmanDecoder {
            s: 0,
            encoded,
            i: 7,
            current: None,
            huffman: &self,
        }
    }
}

struct HuffmanDecoder<'a, T: Iterator<Item = u8>> {
    s: usize,
    encoded: T,
    i: u8,
    current: Option<u8>,
    huffman: &'a HuffmanTree,
}

impl<'a, T: Iterator<Item = u8>> Iterator for HuffmanDecoder<'a, T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_none() {
            self.current = dbg!(self.encoded.next());
        }
        if let Some(byte) = self.current {
            let b = ((1u8 << self.i & byte) != 0) as usize;
            dbg!(b.clone());
            if self.i == 0 {
                self.current = None;
                self.i = 7;
            } else {
                self.i -= 1;
            }

            match self.huffman.tree.get(self.s) {
                Some(Node::Value(v)) => {
                    self.s = 0;
                    Some(*v)
                }
                Some(Node::Split(x)) => {
                    self.s = b + *x as usize;
                    self.next()
                }
                None => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn decode_0() {
    //     use Node::*;
    //     let huffman = HuffmanTree {
    //         tree: vec![Split(1), Split(3), Split(5), Value(1)],
    //     };
    //     let mut v = Vec::new();
    //     huffman.decode(&mut [0b0011_1111u8].iter(), &mut v);
    //     assert_eq!(vec![1], v);
    // }

    #[test]
    fn compose_0() {
        let mut tree = HuffmanTree::default();
        tree.compose(&[0, 1], &[1]);
        use Node::*;
        let expected = vec![Split(1), Split(3), Split(5), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn compose_1() {
        let mut tree = HuffmanTree::default();
        tree.compose(&[1], &[1]);
        use Node::*;
        let expected = vec![Split(1), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn compose_5() {
        let mut tree = HuffmanTree::default();
        let counting = [0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let symbol = [5, 6, 3, 4, 2, 7, 8, 1, 0, 9];

        tree.compose(&counting, &symbol);

        let encoding = vec![
            (0b0011_1111, 5),
            (0b0111_1111, 6),
            (0b1001_1111, 3),
            (0b1011_1111, 4),
            (0b1100_1111, 2),
            (0b1101_1111, 7),
            (0b1110_1111, 8),
            (0b1111_0111, 1),
            (0b1111_1011, 0),
            (0b1111_1101, 9),
        ];

        for (value, expected) in encoding.into_iter() {
            let v: Vec<u8> = vec![value];
            let res: Vec<_> = tree.decode(v.into_iter()).collect();
            assert_eq!(res, vec![expected]);
        }
    }

    #[test]
    fn compose_2() {
        let counts = &[0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[5, 6, 3, 4, 2, 7, 8, 1, 0, 9];
        let mut tree = HuffmanTree::default();
        tree.compose(counts, elements);
        use Node::*;
        let expected: Vec<Node> = vec![
            Split(1),
            Split(3),
            Split(5),
            Value(5),
            Value(6),
            Split(7),
            Split(9),
            Value(3),
            Value(4),
            Split(11),
            Split(13),
            Value(2),
            Value(7),
            Value(8),
            Split(15),
            Value(1),
            Split(17),
            Value(0),
            Split(19),
            Value(9),
        ];
        assert_eq!(expected, tree.tree);
    }
}

#[cfg(test)]
mod benches {
    use super::*;

    use test::{black_box, Bencher};

    #[bench]
    fn compose_1(b: &mut Bencher) {
        let counts = &[0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[5, 6, 3, 4, 2, 7, 8, 1, 0, 9];
        let mut tree = HuffmanTree::default();
        tree.compose(counts, elements);
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }

    #[bench]
    fn compose_2(b: &mut Bencher) {
        let counts = &[0, 2, 1, 3, 2, 4, 5, 2, 4, 4, 3, 4, 8, 5, 5, 1];
        let elements = &[
            1, 2, 3, 0, 4, 17, 5, 33, 6, 18, 49, 65, 7, 19, 34, 81, 97, 20, 113, 8, 50, 129, 145,
            21, 35, 66, 161, 82, 177, 193, 51, 98, 209, 225, 9, 22, 23, 36, 114, 146, 240, 241, 37,
            52, 67, 130, 178, 24, 39, 68, 83, 162, 115,
        ];
        let mut tree = HuffmanTree::default();
        tree.compose(counts, elements);
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }

    #[bench]
    fn compose_3(b: &mut Bencher) {
        let counts = &[0, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[1, 2, 0, 3, 4, 5, 6, 7];
        let mut tree = HuffmanTree::default();
        tree.compose(counts, elements);
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }

    #[bench]
    fn compose_4(b: &mut Bencher) {
        let counts = &[0, 2, 2, 2, 2, 2, 1, 3, 3, 1, 7, 4, 2, 3, 0, 0];
        let elements = &[
            0, 1, 2, 17, 3, 33, 18, 49, 4, 65, 81, 19, 34, 97, 5, 50, 113, 145, 20, 35, 66, 129,
            161, 177, 209, 6, 21, 193, 240, 36, 241, 51, 82, 162,
        ];
        let mut tree = HuffmanTree::default();
        tree.compose(counts, elements);
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }
}
