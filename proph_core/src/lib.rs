#![feature(test)]
extern crate test;

const START_OF_IMAGE: u16 = 0xffd8;
const APPLICATION_DEFAULT_HEADER: u16 = 0xffe0;
const QUANTIZATION_TABLE: u16 = 0xffdb;
const START_OF_FRAME: u16 = 0xffc0;
const DEFINE_HUFFMAN_TABLE: u16 = 0xffc4;
const START_OF_SCAN: u16 = 0xffda;
const END_OF_IMAGE: u16 = 0xffd9;

pub fn decode(f: &[u8]) {
    let mut index = 0;

    loop {
        let marker = u16::from_be_bytes([f[index], f[index + 1]]);
        index += 2;
        print!("marker {:#04x}", marker);
        print!(" {}", marker_name(marker));
        println!();
        match marker {
            START_OF_IMAGE => (),
            END_OF_IMAGE => break,
            START_OF_SCAN => index = f.len() - 2,
            _ => {
                let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);

                let final_index = index + lenchunk as usize;
                if marker == DEFINE_HUFFMAN_TABLE {
                    decode_huffman(&f[index + 2..final_index]);
                }
                index = final_index;
            }
        };
    }
}

fn decode_huffman(f: &[u8]) {
    let mut off = 0;
    let header = f[off];
    off += 1;

    // # Extract the 16 bytes containing length data
    let lengths = f.iter().skip(off).take(16).cloned().collect::<Vec<_>>();
    off += 16;

    // # Extract the elements after the initial 16 bytes
    let mut elements: Vec<u8> = Vec::new();

    for i in lengths.iter().cloned() {
        elements.append(
            &mut f
                .iter()
                .skip(off)
                .take(i as usize)
                .cloned()
                .collect::<Vec<_>>(),
        );

        off += i as usize;
    }

    println!("Header: {}", header);
    println!("lengths: {:?}", lengths);
    println!("Elements: {}", elements.len());
    let mut huffman = HuffmanTree::new();
    huffman.compose(&lengths[..], &elements[..]);
    println!("Huffman tree {:?}", huffman.tree);
}

fn marker_name(marker: u16) -> &'static str {
    match marker {
        START_OF_IMAGE => "Start of Image",
        APPLICATION_DEFAULT_HEADER => "Application Default Header",
        QUANTIZATION_TABLE => "Quantization Table",
        START_OF_FRAME => "Start of Frame",
        DEFINE_HUFFMAN_TABLE => "Define Huffman Table",
        START_OF_SCAN => "Start of Scan",
        END_OF_IMAGE => "End of Image",
        _ => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Node {
    Value(u8),
    Split(usize),
    None,
}

pub struct HuffmanTree {
    tree: Vec<Node>,
}

impl HuffmanTree {
    pub fn new() -> Self {
        Self { tree: Vec::new() }
    }
    pub fn compose(&mut self, counting: &[u8], symbol: &[u8]) {
        let last_index = counting
            .iter()
            .rposition(|val| *val != 0)
            .unwrap_or(counting.len() - 1)
            + 1;

        self.tree.clear();
        self.tree.push(Node::Split(1));

        let mut used = 0;
        let mut s = 0;
        let mut n = 3;

        for index in 0..last_index {
            let c = counting[index];

            for _ in 0..c {
                self.tree.push(Node::Value(symbol[s]));
                s += 1;
            }
            if index == last_index - 1 {
                break;
            }

            used = 2 * used + c as u32;
            let link = 2u32.pow(index as u32 + 1) - used;

            for _ in 0..link {
                self.tree.push(Node::Split(n));
                n += 2;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let f = include_bytes!("profile.jpg");
        decode(f);
    }

    #[test]
    fn test_huffman_0() {
        let mut tree = HuffmanTree::new();
        tree.compose(&[0, 1], &[1]);
        use Node::*;
        let expected = vec![Split(1), Split(3), Split(5), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn test_huffman_1() {
        let mut tree = HuffmanTree::new();
        tree.compose(&[1], &[1]);
        use Node::*;
        let expected = vec![Split(1), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn test_huffman_2() {
        let counts = &[0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[5, 6, 3, 4, 2, 7, 8, 1, 0, 9];
        let mut tree = HuffmanTree::new();
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

    use test::{black_box, Bencher};

    #[bench]
    fn bench_huffman_one(b: &mut Bencher) {
        let counts = &[0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[5, 6, 3, 4, 2, 7, 8, 1, 0, 9];
        let mut tree = HuffmanTree::new();
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }

    #[bench]
    fn bench_huffman_two(b: &mut Bencher) {
        let counts = &[0, 2, 1, 3, 2, 4, 5, 2, 4, 4, 3, 4, 8, 5, 5, 1];
        let elements = &[
            1, 2, 3, 0, 4, 17, 5, 33, 6, 18, 49, 65, 7, 19, 34, 81, 97, 20, 113, 8, 50, 129, 145,
            21, 35, 66, 161, 82, 177, 193, 51, 98, 209, 225, 9, 22, 23, 36, 114, 146, 240, 241, 37,
            52, 67, 130, 178, 24, 39, 68, 83, 162, 115,
        ];
        let mut tree = HuffmanTree::new();
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }

    #[bench]
    fn bench_huffman_three(b: &mut Bencher) {
        let counts = &[0, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[1, 2, 0, 3, 4, 5, 6, 7];
        let mut tree = HuffmanTree::new();
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }

    #[bench]
    fn bench_huffman_four(b: &mut Bencher) {
        let counts = &[0, 2, 2, 2, 2, 2, 1, 3, 3, 1, 7, 4, 2, 3, 0, 0];
        let elements = &[
            0, 1, 2, 17, 3, 33, 18, 49, 4, 65, 81, 19, 34, 97, 5, 50, 113, 145, 20, 35, 66, 129,
            161, 177, 209, 6, 21, 193, 240, 36, 241, 51, 82, 162,
        ];
        let mut tree = HuffmanTree::new();
        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(tree.compose(counts, elements));
            }
        });
    }
}
