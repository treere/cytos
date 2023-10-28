#![feature(test)]
extern crate test;

mod huffman;
mod huffman_old;

use std::collections::HashMap;

pub use huffman::HuffmanTree;
pub use huffman_old::HuffmanTree as HuffmanTreeOld;

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
}

impl Decoder {
    pub fn load(&mut self, f: &[u8]) {
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
                        self.decode_huffman(&f[index + 2..final_index]);
                    }
                    if marker == QUANTIZATION_TABLE {
                        self.decode_quantization(&f[index + 2..final_index]);
                    }
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
            .or_insert(HuffmanTree::default())
            .compose(&lengths[..], &elements[..]);
    }

    fn decode_quantization(&mut self, f: &[u8]) {
        let mut f = f.iter();
        let header = *f.next().unwrap();
        println!("hdr {:#04x}", header);

        let quant = f.take(64).cloned().collect::<Vec<_>>();
        println!("quantization {:?}", quant);

        self.quant.insert(header, quant);
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
        _ => unreachable!(),
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
