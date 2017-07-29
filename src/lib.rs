use std::io;
use std::io::Read;

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
    unknown2: Vec<u8>,
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

        let unknown1 = slice.read_u32::<BigEndian>()?;
        let dialogue_header_offset = slice.read_u32::<BigEndian>()?;

        let mut unknown2 = vec![0; dialogue_header_offset as usize - 8];
        slice.read_exact(&mut unknown2)?;
        let number_of_dialogue_entries = slice.read_u32::<BigEndian>()?;
        let dialogue_offset_table_offset = slice.read_u32::<BigEndian>()?;

        return Ok(MapTable {
            unknown1: unknown1,
            dialogue_header_offset: dialogue_header_offset,
            unknown2: unknown2,
            number_of_dialogue_entries: number_of_dialogue_entries,
            dialogue_offset_table_offset: dialogue_offset_table_offset,
        });
    }
}

impl DialogueOffsetTable {
    pub fn parse(data : &[u8], length : u32) -> io::Result<DialogueOffsetTable> {
        // The length isn't strictly necessary given we can calculate it
        // from the length of the data section passed, but might as well
        // verify it's the case.
        debug_assert!(data.len() / 4 == length as usize);

        let mut data_copy = vec![0; data.len()];
        data_copy.copy_from_slice(data);
        let mut slice = data_copy.as_slice();

        let mut offsets = vec![];

        for _ in 0..length {
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
                // 0x08 is the string terminator; usually two bytes as 0x0800
                .take_while(|c| **c != 0x08)
                // Strings shouldn't have NUL bytes because of the above, but
                // at least some pointers are pointing into padding data that
                // doesn't end in 0x08 but does contain long series of 0x00s.
                // We should strip those to avoid confusing CSV parsers.
                .take_while(|c| **c != 0x00)
                .collect::<Vec<&u8>>());
            let (cow, _, _) = SHIFT_JIS.decode(&string);
            let unicode_string = cow.into_owned()
                                    // The one character different between standard SJIS,
                                    // and the SJIS used by this game.
                                    .replace("曖", "❤");
            dialogue.push(Dialogue {
                chunk: format!("{}", chunk),
                offset: format!("{:#X}", offset),
                character: String::from(""),
                expression: String::from(""),
                japanese: unicode_string,
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
