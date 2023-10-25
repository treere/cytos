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
        println!("");
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
    let huffman = HuffmanTree::new(&lengths[..], &elements[..]);
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
    pub fn new(counting: &[u8], symbol: &[u8]) -> Self {
        // Create a tree with only the root and 2 empty nodes
        let mut root = vec![Node::Split(1), Node::None, Node::None];
        // Left most index
        let mut left_most = 1;
        // Offset in symbol table
        let mut symbol_offset = 0;
        // Last written value
        let mut last_value = 0;
        // Last not zero value in counting array
        let last_index = counting
            .iter()
            .enumerate()
            .rfind(|(_index, val)| **val != 0)
            .map(|(index, _val)| index)
            .unwrap_or(counting.len() - 1)
            + 1;

        for count in counting.iter().take(last_index) {
            // Where the level ends
            let end = root.len();

            // Setting values to the leaf nodes
            for index in 0..*count {
                root[left_most] = Node::Value(symbol[(index + symbol_offset) as usize]);
                last_value = left_most;
                left_most += 1;
            }

            // Saving offset
            symbol_offset += *count;

            let to_add = end - left_most;
            // Add split nodes that points to the new level
            for i in 0..to_add {
                root[left_most + i] = Node::Split(end + 2 * i);
            }

            // Add level nodes
            root.extend([Node::None].iter().cycle().take(2 * to_add));

            // Leftmost node is the 1st of the new layer
            left_most = end;
        }

        Self {
            tree: root.into_iter().take(last_value + 1).collect(),
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
        let tree = HuffmanTree::new(&[0, 1], &[1]);
        use Node::*;
        let expected = vec![Split(1), Split(3), Split(5), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn test_huffman_1() {
        let tree = HuffmanTree::new(&[1], &[1]);
        use Node::*;
        let expected = vec![Split(1), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn test_huffman_2() {
        let counts = &[0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[5, 6, 3, 4, 2, 7, 8, 1, 0, 9];
        let tree = HuffmanTree::new(counts, elements);
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

        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(HuffmanTree::new(counts, elements));
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

        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(HuffmanTree::new(counts, elements));
            }
        });
    }

    #[bench]
    fn bench_huffman_three(b: &mut Bencher) {
        let counts = &[0, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[1, 2, 0, 3, 4, 5, 6, 7];

        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(HuffmanTree::new(counts, elements));
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

        b.iter(|| {
            // Inner closure, the actual test
            for _ in 1..1000 {
                black_box(HuffmanTree::new(counts, elements));
            }
        });
    }
}
