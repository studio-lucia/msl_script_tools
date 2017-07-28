use std::io;

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

extern crate csv;

extern crate encoding_rs;
use encoding_rs::SHIFT_JIS;

#[macro_use]
extern crate serde_derive;

pub struct MapTable {
    #[allow(dead_code)]
    unknown1: u32,
    pub dialogue_header_offset: u32,
    #[allow(dead_code)]
    unknown2: u32,
    pub number_of_dialogue_entries: u32,
    pub dialogue_offset_table_offset: u32,
}

pub struct DialogueOffsetTable {
    offsets: Vec<u32>,
}

impl MapTable {
    pub fn parse(data : &[u8]) -> io::Result<MapTable> {
        // read methods are destructive; we copy the data here
        // to avoid mutating the original data structure.
        let mut data_copy = vec![0; data.len()];
        data_copy.copy_from_slice(data);
        let mut slice = data_copy.as_slice();

        return Ok(MapTable {
            unknown1: slice.read_u32::<BigEndian>()?,
            dialogue_header_offset: slice.read_u32::<BigEndian>()?,
            unknown2: slice.read_u32::<BigEndian>()?,
            number_of_dialogue_entries: slice.read_u32::<BigEndian>()?,
            dialogue_offset_table_offset: slice.read_u32::<BigEndian>()?,
        });
    }
}

impl DialogueOffsetTable {
    pub fn parse(data : &[u8], length : u32) -> io::Result<DialogueOffsetTable> {
        let mut data_copy = vec![0; data.len()];
        data_copy.copy_from_slice(data);
        let mut slice = data_copy.as_slice();

        let mut offsets = vec![];

        debug_assert!(length % 4 == 0);
        for _ in 1..length / 4 {
            offsets.push(slice.read_u32::<BigEndian>()?);
        }
        return Ok(DialogueOffsetTable {
            offsets: offsets,
        });
    }

    pub fn extract_lines(self, data : &[u8], chunk : usize) -> io::Result<Vec<Dialogue>> {
        let mut dialogue = vec![];

        for offset in self.offsets {
            let range = offset as usize..data.len();
            let mut string = vec![];
            string.extend(data[range]
                .iter()
                .take_while(|c| **c != 0x08)
                .collect::<Vec<&u8>>());
            let (cow, _, _) = SHIFT_JIS.decode(&string);
            dialogue.push(Dialogue {
                chunk: format!("{}", chunk),
                offset: format!("{:#X}", offset),
                character: String::from(""),
                expression: String::from(""),
                japanese: cow.into_owned(),
                english: String::from(""),
            });
        }

        return Ok(dialogue);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dialogue {
    // We gather together all the chunks of a map into one script dump,
    // so we want to track the chunk a given bit of dialogue is from.
    chunk: String,
    // Offset within this chunk.
    offset: String,
    // Who's speaking this line? Often unidentifiable, but if we can
    // parse this from the character portrait, it's useful metadata.
    character: String,
    // If we've figured out who's speaking, we also know which
    // portrait is being used, so we can document their expression.
    expression: String,
    // Decoded from Shift JIS.
    japanese: String,
    // This defaults to empty for obvious reasons.
    english: String,
}
