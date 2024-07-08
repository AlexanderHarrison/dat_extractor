# DAT Extractor

A fast and portable alternative to HSDRaw. Used in Rwing.

## Usage

It is not recommended to use this library. This is not meant to be a general purpose library; I only add features as I need them in Rwing.
the public API is quite messy and spread across files, with little to no documentation.
Use HSDRaw if you need functionality right now and don't require dat_extractor's much better performance.

That being said, if you have need of some feature fill out an issue and I might be able add it.
PRs would speed this along.

## Organization

The crate has numerous examples. Some of these are useful tools, like `read_files`, `replace_file`, and `alter_anim_speed`,
while the rest are just explorative and used for debugging.

`isoparser.rs` contains code for reading and caching the dat files from an iso file.
This can be useful as a reference, and for more complex and performant usage than GCRebuilder.
It also supports writing dat files if the replacement file is the same length or shorter than the original.

The source contains two main modules, `dat` and `repr`.
These are two separate reimplementations of HSDRaw.
You should focus on `dat`. It's the most complete and the easiest to read and use.
It is performant enough for almost any usecase.

`repr` is a work in progress, aimed at reducing the complexity of HSDRaw.
`dat` inherited the complexity of the `HSDRawFile` class due to my lack of understanding dat files when started this crate.
Turns out, if you don't care about safety or explorative work, and purely care about parsing and simplicity, you can eliminate heaps of complexity.
HSDRaw, for example, turns all references into a HashMap of 'offset' -> 'struct object'.
This is useful, but slow. If you keep all references as u32 indices into the parent dat file, then you can completely avoid the creation of a HashMap.
HSDRaw also parses the lengths of the structs. Because each struct has a definite length, you don't need to compute these.
I am also experimenting with introducing a bump allocation scheme to avoid heap allocations, but I might not take this route.
