# RIFFU

[![Crate](https://img.shields.io/crates/v/riffu.svg)](https://crates.io/crates/riffu)

The Resource Interchange File Format (RIFF) is a generic file container format for storing data in tagged chunks.
It is primarily used to store multimedia such as sound and video, though it may also be used to store any arbitrary data.

## Crate for doing IO on RIFF-formatted files

This crate provides utility methods for reading and writing formats such as
Microsoft Wave, Audio Video Interleave or Downloadable Sounds.

### Examples

Please see the [tests](tests) to see how to use it.

### TODO

I plan to add many, many features to this crate.
I hope it will become the de facto way to parse RIFF files in Rust :) -- one can dream.

I am planning on working on:

1. ~~Complete propagation of errors.
A library ought to never panics.
Every possible errors should propagate back to the user of this library in `RiffError`.
This also means that `RiffError` needs to be a LOT more robust.~~

2. ~~Ability to dynamically construct RIFF files and write to the machine.~~

3. A clean implementation of the lazy version.
Because a type `T` that satisfy `Read + Seek` must be mutable to do anything.
Getting this to work nicely is quite hard because only 1 mutable reference to `T` can exist at any one time.
However, because we are recursively parsing through the file, it complains that it cannot infer the lifetime of `T`.

4. Unify the interface of the 2 versions under a trait.

5. Conversion from each of these representations.

## Note

This is a fork of the [original library](https://github.com/frabert/riff) with major changes (practically a rewrite of the entire thing).
Provides a cleaner interface, and an option to read the file eagerly.