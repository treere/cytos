#![feature(test)]
extern crate test;

mod huffman;
mod huffman_old;
mod utils;

use std::collections::HashMap;

pub use huffman::HuffmanTree;
pub use huffman_old::HuffmanTree as HuffmanTreeOld;
use utils::RemoveFF00;

use crate::utils::BitIterator;

const START_OF_IMAGE: u16 = 0xffd8;
const APPLICATION_DEFAULT_HEADER: u16 = 0xffe0;
const QUANTIZATION_TABLE: u16 = 0xffdb;
const START_OF_FRAME: u16 = 0xffc0;
const DEFINE_HUFFMAN_TABLE: u16 = 0xffc4;
const START_OF_SCAN: u16 = 0xffda;
const END_OF_IMAGE: u16 = 0xffd9;

#[derive(Default)]
pub struct Decoder {
    precision: u8,
    size: (u16, u16), // r c
    components: Vec<(u8, u8, u8, u8)>,
    huffman: HashMap<u8, HuffmanTree>,
    quant: HashMap<u8, Vec<u8>>,
}

impl Decoder {
    pub fn print(&mut self) {
        println!("Size: height {} width {}", self.size.0, self.size.1);
        println!("Precision: {}", self.precision);
        println!("Components: {}", self.components.len());
        for (id, vs, hs, qtbid) in self.components.iter() {
            println!(
                "   id: {}, samp_vert: {}, samp_hori: {}, qtbit: {}",
                id, vs, hs, qtbid
            )
        }
        println!("-- quantization --");
        for (quant, _) in self.quant.iter() {
            println!("   id: {}", quant);
        }
        println!("-- huffman --");
        for (huffman, _) in self.huffman.iter() {
            println!("   id: {}", huffman);
        }
    }
    pub fn load(&mut self, f: &[u8]) {
        let mut index = 0;

        loop {
            let marker = u16::from_be_bytes([f[index], f[index + 1]]);
            index += 2;
            match marker {
                START_OF_IMAGE => (),
                START_OF_FRAME => {
                    let (chunk, final_index) = Self::take_chunk(f, index);
                    index = final_index;
                    self.decode_start_of_frame(chunk);
                }

                START_OF_SCAN => {
                    let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]) as usize;
                    let len = self.decode_start_of_scan(&f[index + lenchunk..]);
                    return;
                    // index = lenchunk + len + 2;-
                }
                DEFINE_HUFFMAN_TABLE => {
                    let (chunk, final_index) = Self::take_chunk(f, index);
                    index = final_index;

                    self.decode_huffman(chunk);
                }
                QUANTIZATION_TABLE => {
                    let (chunk, final_index) = Self::take_chunk(f, index);
                    index = final_index;

                    self.decode_quantization(chunk);
                }
                END_OF_IMAGE => break,
                _ => {
                    let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);

                    let final_index = index + lenchunk as usize;

                    index = final_index;
                }
            };
        }
    }

    fn decode_huffman(&mut self, f: &[u8]) {
        let header = f[0];
        let _index = header & 0b0000_1111;
        let _htype = (header & 0b0001_0000) >> 4;

        // # Extract the 16 bytes containing length data
        let lengths = &f[1..1 + 16];

        let total = lengths.iter().fold(0, |a, b| a + *b as u32) as usize;
        let elements = &f[1 + 16..1 + 16 + total];

        self.huffman
            .entry(header)
            .or_default()
            .compose(lengths, elements);
    }

    fn decode_quantization(&mut self, f: &[u8]) {
        let mut f = f.iter();
        let header = *f.next().unwrap();

        let quant = f.take(64).cloned().collect::<Vec<_>>();

        self.quant.insert(header, quant);
    }

    fn decode_start_of_frame(&mut self, f: &[u8]) {
        self.precision = f[0];
        let height = u16::from_be_bytes([f[1], f[2]]);
        let width = u16::from_be_bytes([f[3], f[4]]);
        self.size = (height, width);
        let components = f[5] as usize;

        self.components.clear();
        for i in 0..components {
            let id = f[6 + i * 3];
            let samp = f[7 + i * 3];
            let samp_vert = 0b0000_1111 & samp;
            let samp_hori = (0b1111_0000 & samp) >> 4;
            let qtbid = f[8 + i * 3];

            self.components.push((id, samp_vert, samp_hori, qtbid));
        }
    }

    fn decode_start_of_scan(&mut self, f: &[u8]) -> usize {
        let iterator = RemoveFF00::new(f);
        let mut iterator = BitIterator::new(iterator);

        // Using 0 but I should read from f
        let code = {
            let h = &self.huffman[&0];

            h.decode(&mut iterator).next()
        }
        .unwrap();
        dbg!(code);
        for _ in 0..code {
            dbg!(iterator.next());
        }

        // dbg!(res[0]);
        let mut iterator = RemoveFF00::new(f);
        for _ in iterator.by_ref() {}

        iterator.len()
    }

    fn take_chunk(f: &[u8], index: usize) -> (&[u8], usize) {
        let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);
        let final_index = index + lenchunk as usize;
        let chunk = &f[index + 2..final_index];

        (chunk, final_index)
    }
}

pub fn marker_name(marker: u16) -> &'static str {
    match marker {
        START_OF_IMAGE => "Start of Image",
        APPLICATION_DEFAULT_HEADER => "Application Default Header",
        QUANTIZATION_TABLE => "Quantization Table",
        START_OF_FRAME => "Start of Frame",
        DEFINE_HUFFMAN_TABLE => "Define Huffman Table",
        START_OF_SCAN => "Start of Scan",
        END_OF_IMAGE => "End of Image",
        _ => unreachable!("{:#04x}", marker),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn it_works() {
        let mut decoder = Decoder::default();
        // let f = include_bytes!("image.jpg");
        let f = include_bytes!("profile.jpg");
        decoder.load(f);
        decoder.print();
    }
}

#[cfg(test)]
mod benches {
    use super::*;

    use test::{black_box, Bencher};

    #[bench]
    #[ignore]
    fn load(b: &mut Bencher) {
        let mut decoder = Decoder::default();
        let f = include_bytes!("image.jpg");
        // let f = include_bytes!("profile.jpg");
        b.iter(|| {
            for _ in 1..1000 {
                black_box(decoder.load(f));
            }
        })
    }
}
