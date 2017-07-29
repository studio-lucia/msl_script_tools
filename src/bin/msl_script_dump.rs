use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

extern crate clap;
use clap::{Arg, App};

extern crate csv;

extern crate fldtools;
use fldtools::ChunkList;

extern crate msl_script_tools;
use msl_script_tools::{Dialogue, DialogueOffsetTable, MapTable};

fn process_file(input_file : &String) -> io::Result<Vec<Dialogue>> {
    let mut dialogue = vec![];

    let mut data = vec![];
    let mut file = File::open(input_file)?;
    file.read_to_end(&mut data)?;
    let chunks = ChunkList::parse(&data[0..2047])?;
    for (i, chunk) in chunks.into_iter().enumerate() {
        // This slice contains the data for just this one chunk,
        // not the whole file.
        // The relative offsets we're using within this portion of
        // the loop are relative to this chunk, not the beginning of
        // the file.
        let start = chunk.start as usize;
        let end = start + chunk.length as usize;

        // TODO: fldtools probably shouldn't be handing out chunks with
        // zero starting positions or zero lengths. But we'll skip for now.
        if chunk.start == 0 {
            println!("Chunk {} starts at an invalid position; skipping", i);
            continue;
        } else if chunk.length == 0 {
            println!("Chunk {} has an invalid length; skipping", i);
            continue;
        }

        let chunk_data = &data[start..end];

        let map_table = MapTable::parse(chunk_data)?;

        let length = map_table.dialogue_offset_table_offset as usize + 
                     (map_table.number_of_dialogue_entries * 4) as usize;
        let range = map_table.dialogue_offset_table_offset as usize..length;

        let dialogue_table = DialogueOffsetTable::parse(&chunk_data[range], map_table.number_of_dialogue_entries)?;

        // OK, now that we have the table, we can start extracting
        // dialogue chunks
        dialogue.extend(dialogue_table.extract_lines(chunk_data, i)?);
    }

    return Ok(dialogue);
}

fn main() {
    let matches = App::new("msl_script_dump")
                          .version("0.2.0")
                          .author("Misty De Meo")
                          .about("Extract Magical School Lunar! script data")
                          .arg(Arg::with_name("input")
                              .help("Script files to process")
                              .required(true)
                              .multiple(true))
                          .arg(Arg::with_name("output")
                              .help("Directory to write script CSVs to")
                              .short("o")
                              .long("output")
                              .takes_value(true))
                          .get_matches();
    let input_files = matches.values_of("input").unwrap().map(|path| String::from(path)).collect::<Vec<String>>();
    if input_files.iter().any(|path| !Path::new(path).exists()) {
        println!("One or more input files couldn't be found!");
        exit(1);
    }

    let output = matches.value_of("output").unwrap_or(".");
    let output_path = Path::new(output);
    if !output_path.exists() {
        println!("Output path does not exist!");
        exit(1);
    }

    for input_file in input_files {
        let dialogue;
        match process_file(&input_file) {
            Ok(lines) => dialogue = lines,
            Err(e) => {
                println!("Error trying to extract dialogue from script {}: {}", input_file, e);
                exit(1);
            }
        }
        println!("Writing output for file: {}", input_file);
        let input_path = Path::new(&input_file);
        let new_filename = format!("{}.csv",
                                   input_path.file_stem().unwrap().to_str().unwrap());
        let output_file_path = output_path.join(new_filename);
        let output_file = File::create(&output_file_path).unwrap();

        let mut writer = csv::Writer::from_writer(output_file);
        for line in dialogue {
            writer.serialize(line).unwrap();
        }
    }
}
