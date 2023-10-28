#[derive(Debug, PartialEq)]
pub enum Node {
    Value(u8),
    Split(usize),
    None,
}

#[derive(Default)]
pub struct HuffmanTree {
    tree: Vec<Node>,
}

impl HuffmanTree {
    pub fn compose(&mut self, counting: &[u8], symbol: &[u8]) {
        // Create a tree with only the root and 2 empty nodes
        self.tree.clear();
        self.tree.push(Node::Split(1));
        self.tree.push(Node::None);
        self.tree.push(Node::None);

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
            let end = self.tree.len();

            // Setting values to the leaf nodes
            for index in 0..*count {
                self.tree[left_most] = Node::Value(symbol[(index + symbol_offset) as usize]);
                last_value = left_most;
                left_most += 1;
            }

            // Saving offset
            symbol_offset += *count;

            let to_add = end - left_most;
            // Add split nodes that points to the new level
            for i in 0..to_add {
                self.tree[left_most + i] = Node::Split(end + 2 * i);
            }

            // Add level nodes
            for _ in 0..to_add * 2 {
                self.tree.push(Node::None);
            }

            // Leftmost node is the 1st of the new layer
            left_most = end;
        }

        self.tree.truncate(last_value + 1);
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
