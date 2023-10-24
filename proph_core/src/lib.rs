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
enum Node {
    Value(u32),
    Split(u32, u32),
    None,
}

fn a(count: &[u8; 16], symbol: &[u8; 16]) {
    let mut root = vec![Node::Split(1, 2), Node::None, Node::None];
    let left_most = 0;
    for c in count.iter() {
        if *c == 0 {
            let current = left_most;
            while root[current] != Node::None {
                let x = root.len();
                root.push(Node::None);
                root.push(Node::None);
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
}
