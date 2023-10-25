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
    println!("Elements: {}", elements.len())
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

#[derive(Debug, PartialEq)]
pub enum Node {
    Value(u32),
    Split(u32, u32),
    None,
}

struct Point {
    level: usize,
    value: Node,
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Node::Value(x) => write!(f, "<{}, {}>", self.level, x),
            Node::Split(x, y) => write!(f, "<{}, {} {}>", self.level, x, y),
            Node::None => write!(f, "<{}, -->", self.level),
        }
    }
}

pub struct HuffmanTree {
    tree: Vec<Node>,
}

impl HuffmanTree {
    pub fn new(count: &[u8], symbol: &[u32]) -> Self {
        let mut last_value = 0;
        let mut root = vec![
            Point {
                level: 0,
                value: Node::Split(1, 2),
            },
            Point {
                level: 1,
                value: Node::None,
            },
            Point {
                level: 1,
                value: Node::None,
            },
        ];
        let mut left_most = 1;
        let mut symbol_offset = 0;

        for (l, c) in count.iter().enumerate() {
            let level = l + 1;

            if *c == 0 {
                let mut current = left_most;
                let end = root.len();
                while current < end {
                    if root[current].level != level {
                        current += 1;
                        continue;
                    }
                    let pos = root.len();
                    root[current].value = Node::Split(pos as u32, (pos + 1) as u32);
                    root.push(Point {
                        level: level + 1,
                        value: Node::None,
                    });
                    root.push(Point {
                        level: level + 1,
                        value: Node::None,
                    });
                    current += 1;
                }
                left_most = end;
            } else {
                let end = root.len();
                for i in 0..*c {
                    root[left_most].value = Node::Value(symbol[(i + symbol_offset) as usize]);
                    last_value = left_most;
                    left_most += 1;
                    while left_most < end {
                        if root[left_most].level == level {
                            break;
                        }
                        left_most += 1;
                    }
                }

                symbol_offset += *c;
                if left_most >= end {
                    unreachable!()
                }
                let pos = root.len();

                root[left_most].value = Node::Split(pos as u32, (pos + 1) as u32);
                root.push(Point {
                    level: level + 1,
                    value: Node::None,
                });
                root.push(Point {
                    level: level + 1,
                    value: Node::None,
                });

                let mut current = left_most + 1;
                left_most = pos;

                while current < end {
                    if root[current].level != level {
                        current += 1;
                        continue;
                    }
                    let pos = root.len();
                    root[current].value = Node::Split(pos as u32, (pos + 1) as u32);
                    root.push(Point {
                        level: level + 1,
                        value: Node::None,
                    });
                    root.push(Point {
                        level: level + 1,
                        value: Node::None,
                    });
                    current += 1;
                }
            }
        }

        Self {
            tree: root
                .into_iter()
                .take(last_value + 1)
                .map(|x| x.value)
                .collect(),
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
        let expected = vec![Split(1, 2), Split(3, 4), Split(5, 6), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn test_huffman_1() {
        let tree = HuffmanTree::new(&[1], &[1]);
        use Node::*;
        let expected = vec![Split(1, 2), Value(1)];
        assert_eq!(expected, tree.tree);
    }

    #[test]
    fn test_huffman_2() {
        let counts = &[0, 2, 2, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let elements = &[5, 6, 3, 4, 2, 7, 8, 1, 0, 9];
        let tree = HuffmanTree::new(counts, elements);
        use Node::*;
        let expected: Vec<Node> = vec![
            Split(1, 2),
            Split(3, 4),
            Split(5, 6),
            Value(5),
            Value(6),
            Split(7, 8),
            Split(9, 10),
            Value(3),
            Value(4),
            Split(11, 12),
            Split(13, 14),
            Value(2),
            Value(7),
            Value(8),
            Split(15, 16),
            Value(1),
            Split(17, 18),
            Value(0),
            Split(19, 20),
            Value(9),
        ];
        assert_eq!(expected, tree.tree);
    }
}
