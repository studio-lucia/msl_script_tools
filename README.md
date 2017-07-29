# msl_script_tools

This set of tools supports manipulating dialogue script data from the game [*Magical School Lunar!*](https://en.wikipedia.org/wiki/Lunar:_Samposuru_Gakuen) (Sega Saturn, 1997). Aside from the commandline tool, this project also contains a Rust crate with a few helpers to let you write your own tools for working with script files.

## Installation

On Mac:

```
brew install mistydemeo/lunar/msl_script_tools
```

Building manually:

Clone this repo, and then

```
make
```

## Usage

### msl_script_dump

Extracts script files into CSV files with the following fields:

* chunk - The index of the chunk in which the string is located.
* offset - The hex offset of the beginning of the string in the chunk. This is relative to the beginning of the chunk, not the beginning of the file.
* character - The name of the character who's speaking. Currently not supported, so always written as a blank string.
* expression - The expression of the character who's speaking. Currently not supported, so always written as a blank string.
* japanese - The line of dialogue, converted from Shift JIS into UTF-8.
* english - Left blank to allow room for translation.

```
msl_script_dump <script_files.fld>
```

For each input file, a CSV file will be written with the same name and the .csv extension. By default, the script files will be written into the working directory; use the `--output` option to choose another directory.
