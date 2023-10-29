#![feature(test)]
extern crate test;

mod huffman;
mod huffman_old;
mod utils;

use std::collections::HashMap;

pub use huffman::HuffmanTree;
pub use huffman_old::HuffmanTree as HuffmanTreeOld;
use utils::RemoveFF00;

const START_OF_IMAGE: u16 = 0xffd8;
const APPLICATION_DEFAULT_HEADER: u16 = 0xffe0;
const QUANTIZATION_TABLE: u16 = 0xffdb;
const START_OF_FRAME: u16 = 0xffc0;
const DEFINE_HUFFMAN_TABLE: u16 = 0xffc4;
const START_OF_SCAN: u16 = 0xffda;
const END_OF_IMAGE: u16 = 0xffd9;

#[derive(Default)]
pub struct Decoder {
    huffman: HashMap<u8, HuffmanTree>,
    quant: HashMap<u8, Vec<u8>>,
    precision: u8,
    size: (u16, u16),
    components: Vec<(u8, u8, u8, u8)>,
}

impl Decoder {
    pub fn load(&mut self, f: &[u8]) {
        let mut index = 0;

        loop {
            let marker = u16::from_be_bytes([f[index], f[index + 1]]);
            index += 2;
            println!(
                "******************** {:#04x} - {} ********************",
                marker,
                marker_name(marker)
            );
            match marker {
                START_OF_IMAGE => (),
                END_OF_IMAGE => break,
                START_OF_SCAN => {
                    let chunk = &f[index + 2..];
                    let len = self.decode_start_of_scan(chunk);
                    index += len + 2;
                }
                DEFINE_HUFFMAN_TABLE => {
                    let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);
                    println!("Length: {}", lenchunk);

                    let final_index = index + lenchunk as usize;
                    let chunk = &f[index + 2..final_index];

                    self.decode_huffman(chunk);
                    index = final_index;
                }
                QUANTIZATION_TABLE => {
                    let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);
                    println!("Length: {}", lenchunk);

                    let final_index = index + lenchunk as usize;
                    let chunk = &f[index + 2..final_index];

                    self.decode_quantization(chunk);
                    index = final_index;
                }
                START_OF_FRAME => {
                    let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);
                    println!("Length: {}", lenchunk);

                    let final_index = index + lenchunk as usize;
                    let chunk = &f[index + 2..final_index];

                    self.decode_start_of_frame(chunk);

                    index = final_index;
                }
                _ => {
                    let lenchunk = u16::from_be_bytes([f[index], f[index + 1]]);
                    println!("Length: {}", lenchunk);

                    let final_index = index + lenchunk as usize;

                    index = final_index;
                }
            };
        }
    }

    fn decode_huffman(&mut self, f: &[u8]) {
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
        self.huffman
            .entry(header)
            .or_default()
            .compose(&lengths[..], &elements[..]);
    }

    fn decode_quantization(&mut self, f: &[u8]) {
        let mut f = f.iter();
        let header = *f.next().unwrap();
        println!("Header: {:#04x}", header);

        let quant = f.take(64).cloned().collect::<Vec<_>>();
        println!("QuantizationMatrix: {:?}", quant);

        self.quant.insert(header, quant);
    }

    fn decode_start_of_frame(&mut self, f: &[u8]) {
        self.precision = f[0];
        let height = u16::from_be_bytes([f[1], f[2]]);
        let width = u16::from_be_bytes([f[3], f[4]]);
        self.size = (height, width);
        let components = f[5] as usize;

        println!("DataPrecision: {}", self.precision);
        println!("Components: {}", components);
        println!("Size: {} X {} (WxH)", width, height);

        self.components.clear();
        for i in 0..components {
            let id = f[6 + i * 3];
            let samp = f[7 + i * 3];
            let samp_vert = 0b0000_1111 & samp;
            let samp_hori = (0b1111_0000 & samp) >> 4;
            let qtbid = f[8 + i * 3];

            println!(
                "Component: {}, Id: {}, SampVert: {}, SampHori: {}, Qtbid: {}",
                i, id, samp_vert, samp_hori, qtbid
            );
            self.components.push((id, samp_vert, samp_hori, qtbid));
        }
    }

    fn decode_start_of_scan(&mut self, f: &[u8]) -> usize {
        let mut iterator = RemoveFF00::new(f);
        for _ in iterator.by_ref() {}
        iterator.len()
    }
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
        _ => unreachable!("{:#04x}", marker),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut decoder = Decoder::default();
        // let f = include_bytes!("image.jpeg");
        let f = include_bytes!("profile.jpg");
        decoder.load(f);
    }
}

#[cfg(test)]
mod benches {
    use super::*;

    use test::{black_box, Bencher};

    #[bench]
    fn load(b: &mut Bencher) {
        let mut decoder = Decoder::default();
        let f = include_bytes!("image.jpeg");
        // let f = include_bytes!("profile.jpg");
        b.iter(|| {
            for _ in 1..1000 {
                black_box(decoder.load(f));
            }
        })
    }
}
