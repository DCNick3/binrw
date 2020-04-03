//! A documentation-only module for attributes
//! 
//! # List of attributes
//! 
//! | Attribute | Supports | Description
//! |-----------|------------------|------------
//! | [big](#byteorder) | all | Set the endianness to big endian
//! | [little](#byteorder) | all | Set the endianness to little endian
//! | [magic](#magic) | top-level | At the start of parsing read a value and make sure it is equivelant to a constant value
//! | [assert](#assert) | top-level | After parsing, check if a condition is true and, optionally, return a custom error if false. Allows multiple.
//! | [import](#arguments) | top-level | Define the arguments for parsing the given type
//! | [args](#arguments) | fields | Pass a set of arguments.
//! | [default](#default) | fields | Set a field to the default value for the type
//! | [ignore](#default) | fields | An alias for `default`
//! | [postprocess_now](#postprocessing) | fields | Immediately run [`after_parse`](crate::BinRead::after_parse) after reading
//! | [deref_now](#postprocessing) | fields | Alias for postprocess_now
//! | [restore_position](#restore-position) | fields | Restore the reader position after reading the field
//! | [try](#try) | fields | Attempt to parse a value and store `None` if parsing fails.
//! | [map](#map) | fields | Read a type from the reader and then apply a function to map it to the type to store in the struct
//! | [parse_with](#custom-parsers) | fields | Use a custom parser function for reading from a file
//! | [calc](#calculations) | fields | Compute an expression to store. Can use previously read values.
//! | [count](#count) | fields | Set the length for a vector
//! | [is_little](#byteorder) | fields | Conditionally set the endian to little
//! | [is_big](#byteorder) | fields | Conditionally set the endian to big
//! | [offset](#offset) | fields | Change the offset a [`FilePtr`](crate::FilePtr) is relative to
//! | [if](#condtional-values) | fields | Used on an [`Option<T>`](core::option::Option) to read a value of type `T` only if the condition is met
//! | [pad_before](#padding-and-alignment) | fields | Skip a constant number of bytes forward before reading
//! | [pad_after](#padding-and-alignment) | fields | Skip a constant number of bytes forward after reading
//! | [align_before](#padding-and-alignment) | fields | Skip to the next Nth byte before reading
//! | [align_after](#padding-and-alignment) | fields | Skip to the next Nth byte after reading
//! | [seek_before](#padding-and-alignment) | fields | Passes the given [`SeekFrom`](crate::io::SeekFrom) to [`Seek::seek`](crate::io::Seek::seek)
//! | [pad_size_to](#padding-and-alignment) | fields | Ensures the cursor is at least N bytes after the starting position for this field
//! | [return_all_errors](#enum-errors) | enum-level | Use an error handling type in which enum failures return a [`Vec`](Vec) with an error for every variant
//! 
//! # Byteorder
//! 
//! You can use `big` or `little` at either the struct-level or the field-level in order
//! to override the byte order of values.
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! #[br(little)]
//! struct MyType (
//!     #[br(big)] u32, // will be big endian
//!     u32, // will be little endian
//! );
//! ```
//! The order of precedence is: (from highed to lowest)
//! 1. Field-level
//! 2. Variant-level (for enums)
//! 3. Top-level
//! 4. Configured (i.e. what endianess was passed in)
//! 5. Native endianess
//!
//! binread also offers the ability to conditionally set endianness for when the endianess
//! is described within the data itself using `is_big` or `is_little`:
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead, Debug, PartialEq)]
//! #[br(big)]
//! struct MyType {
//!     val: u8,
//!     #[br(is_little = (val == 3))]
//!     other_val: u16
//! }
//! 
//! # assert_eq!(MyType::read(&mut Cursor::new(b"\x03\x01\x00")).unwrap(), MyType { val: 3, other_val: 1 });
//! ```
//! 
//! # Magic
//! 
//! Magic, or magic values, are constants used for sanity/integrity checking or simply for
//! making file identification easier. Since these are such a common use case binread provides
//! an attribute for handling this for you to save code/memory/time/etc.
//! 
//! The format is `magic = [lit]` where `[lit]` is any literal supported by Rust. This is allowed
//! at the following levels: struct, enum, variant, and field.
//! 
//! **Examples:**
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead, Debug)]
//! #[br(magic = b"TEST")]
//! struct Test {
//!     val: u32
//! }
//! 
//! #[derive(BinRead, Debug)]
//! #[br(magic = 1.2f32)]
//! struct Version(u16);
//! 
//! #[derive(BinRead)]
//! enum Command {
//!     #[br(magic = 0u8)] Nop,
//!     #[br(magic = 1u8)] Jump { loc: u32 },
//!     #[br(magic = 2u8)] Begin { var_count: u16, local_count: u16 }
//! }
//! ```
//! 
//! Example error:
//! ```text
//! Error::BadMagic { pos: 0x30 }
//! ```
//! See [`binread::Error`](crate::Error::BadMagic) for more info.
//!
//! # Assert
//! 
//! `assert` is the core of error handling in BinRead. It returns either an [`AssertFail`](crate::Error::AssertFail)
//! or, optionally, a custom user-generated error, allowing you to attach context from before
//! parsing failed.
//! 
//! **Custom Error Handling Example:**
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(Debug, PartialEq)]
//! struct NotSmallerError(u32, u32);
//!
//! #[derive(BinRead, Debug)]
//! #[br(assert(some_val > some_smaller_val, NotSmallerError(some_val, some_smaller_val)))]
//! struct Test {
//!     some_val: u32,
//!     some_smaller_val: u32
//! }
//!
//! let error = Cursor::new(b"\0\0\0\x01\0\0\0\xFF").read_be::<Test>();
//! assert!(error.is_err());
//! let error = error.unwrap_err();
//! assert_eq!(error.custom_err(), Some(&NotSmallerError(0x1, 0xFF)));
//! ```
//! 
//! # Arguments
//! One feature of binread is allowing arguments to be passed to the type in order to tell
//! the type any info it needs to parse the data. To accept arguments when using the derive
//! macro, you can use the `import` attribute and to pass arguments you can use the `args`
//! attribute.
//! 
//! **Example:**
//! ```rust
//! # use binread::prelude::*;
//! 
//! #[derive(BinRead)]
//! #[br(import(val1: u32, val2: &'static str))]
//! struct ImportTest {
//!     // ...
//! }
//! 
//! #[derive(BinRead)]
//! struct ArgsTets {
//!     val: u32,
//!     #[br(args(val + 3, "test"))]
//!     test: ImportTest
//! }
//! ```
//! # Default
//! 
//! Set the field to the default value for the type.
//! 
//! ```rust
//! # use binread::{BinRead, io::Cursor};
//! #[derive(BinRead, Debug, PartialEq)]
//! struct Test {
//!     #[br(default)]
//!     path: Option<std::path::PathBuf>,
//! }
//! 
//! assert_eq!(
//!     Test::read(&mut Cursor::new(vec![])).unwrap(),
//!     Test { path: None }
//! );
//! ```
//! 
//! # Postprocessing
//! 
//! In binread postprocessing refers to the act of running [`after_parse`](crate::BinRead::after_parse) on
//! a field. It is used in order to allow a field to take control of the reader temporarily in
//! order to parse any values not stored inline. 
//! 
//! Postprocessing can be fast-tracked using either `deref_now` or `postprocess_now` (these are
//! simply aliases for each other to allow). `deref_now` is recommended for [`FilePtr`](crate::FilePtr)'s,
//! `post_process` is recommended for anything else.
//! 
//! ```rust
//! # use binread::{prelude::*, FilePtr32, NullString, io::Cursor};
//! 
//! #[derive(BinRead, Debug)]
//! #[br(big, magic = b"TEST")]
//! struct TestFile {
//!     #[br(deref_now)]
//!     ptr: FilePtr32<NullString>,
//!
//!     value: i32,
//! 
//!     // Notice how `ptr` can be used as it has already been postprocessed
//!     #[br(calc = ptr.len())]
//!     ptr_len: usize,
//! }
//!
//! # let test_contents = b"\x54\x45\x53\x54\x00\x00\x00\x10\xFF\xFF\xFF\xFF\x00\x00\x00\x00\x54\x65\x73\x74\x20\x73\x74\x72\x69\x6E\x67\x00\x00\x00\x00\x69";
//! # let test = Cursor::new(test_contents).read_be::<TestFile>().unwrap();
//! # assert_eq!(test.ptr_len, 11);
//! # assert_eq!(test.value, -1);
//! # assert_eq!(test.ptr.to_string(), "Test string");
//! ```
//! 
//! # Restore Position
//!
//! binread supports restoring the reader position after reading the field using the
//! `restore_position` attribute.
//!  
//! **Example:**
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead, Debug, PartialEq)]
//! struct MyType {
//!     #[br(restore_position)]
//!     test: u32,
//!     test_bytes: [u8; 4]
//! }
//!
//! # assert_eq!(
//! #   Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap(),
//! #   MyType { test: 1, test_bytes: [0,0,0,1]}
//! # );
//! ```
//! 
//! # Try
//! 
//! When you want to optionally allow parsing to fail, wrap the type with [`Option`](std::option::Option)
//! and use the `try` attribute. binread makes no guarantees about the position of the reader
//! after a `try` in which parsing failed, so use with caution.
//! 
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(try)]
//!     maybe_u32: Option<u32>
//! }
//! 
//! assert_eq!(Cursor::new(b"").read_be::<MyType>().unwrap().maybe_u32, None);
//! ```
//! 
//! # Map
//! 
//! Sometimes the form you read isn't the form you want to store. For that, you can use the `map`
//! attribute in order to apply a mapping function to modify it before storage.
//! 
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(map = |x: u8| x.to_string())]
//!     int_str: String
//! }
//! 
//! # assert_eq!(Cursor::new(b"\0").read_be::<MyType>().unwrap().int_str, "0");
//! ```
//!
//! 
//! # Custom Parsers
//!
//! In some cases, you need more advanced logic than deriving BinRead provides. For that,
//! binread provides the `parse_with` attribute to allow specifying custom parser functions.
//! 
//! ```rust
//! # use binread::{prelude::*, io::*, ReadOptions};
//! # use std::collections::HashMap;
//! 
//! fn custom_parser<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ())
//!     -> BinResult<HashMap<u16, u16>>
//! {
//!     let mut map = HashMap::new();
//!     map.insert(
//!         reader.read_be().unwrap(),
//!         reader.read_be().unwrap()
//!     );
//!     Ok(map)
//! }
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(parse_with = custom_parser)]
//!     offsets: HashMap<u16, u16>
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap().offsets.get(&0), Some(&1));
//! ```
//! 
//! This can also be used with [`FilePtr::parse`](crate::FilePtr::parse) in order to read and
//! immediately dereference a [`FilePtr`](crate::FilePtr) to an owned value.
//! 
//! ```rust
//! # use binread::{prelude::*, io::Cursor, FilePtr32, NullString};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(parse_with = FilePtr32::parse)]
//!     some_string: NullString,
//! }
//! 
//! # let val: MyType = Cursor::new(b"\0\0\0\x04Test\0").read_be().unwrap();
//! # assert_eq!(val.some_string.to_string(), "Test");
//! ```
//! 
//! # Calculations
//! 
//! Sometimes you don't want to read a value from a file, but you want to set a field equal to an
//! expression. Or you just want to initialize a value for a type that doesn't have a [`Default`](core::default::Default)
//! implementation.
//! 
//! 
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     var: u32,
//!     #[br(calc = 3 + var)]
//!     var_plus_3: u32,
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap().var_plus_3, 4);
//! ```
//! 
//! # Count
//! 
//! The `count` attribute 
//! 
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     size: u32,
//!     #[br(count = size)]
//!     data: Vec<u8>,
//! }
//!
//! # assert_eq!(
//! #    Cursor::new(b"\0\0\0\x04\x01\x02\x03\x04").read_be::<MyType>().unwrap().data,
//! #    &[1u8, 2, 3, 4]
//! # );
//! ```
//! 
//! # Offset
//! 
//! Sometimes, when you use a [`FilePtr`](crate::FilePtr) you want it to represent a location that
//! is relative to a certain location in the reader other than the start of the reader. For that 
//! you want to use one of two attributes: `offset` and `offset_after`.
//!
//! **Example:**
//! ```rust
//! # use binread::{prelude::*, io::Cursor, FilePtr};
//! 
//! #[derive(BinRead, Debug, PartialEq)]
//! struct OffsetTest {
//!     #[br(little, offset = 4)]
//!     test: FilePtr<u8, u16>
//! }
//!
//! # assert_eq!(
//! #   *OffsetTest::read(&mut Cursor::new(b"\0\xFF\xFF\xFF\x02\0")).unwrap().test,
//! #   2u16
//! # );
//! ```
//!
//! The only time you need to use `offset_after` is if, in your offset calculation, you use
//! fields that are defined after your FilePtr. Otherwise use `offset` as `offset_after` doesn't
//! support some features of binread due to order of execution.
//! 
//! # Conditional Values
//! 
//! binread also provides the ability to conditionally parse an [`Option<T>`](Option) field
//! using the `if` attribute. If the condition is true it will parse the value and store it as
//! `Some(T)`, otherwise it will store `None`.
//! 
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     var: u32,
//!
//!     #[br(if(var == 1))]
//!     original_byte: Option<u8>,
//!
//!     #[br(if(var != 1))]
//!     other_byte: Option<u8>,
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01\x03").read_be::<MyType>().unwrap().original_byte, Some(3));
//! # assert_eq!(Cursor::new(b"\0\0\0\x01\x03").read_be::<MyType>().unwrap().other_byte, None);
//! ```
//! 
//! # Padding and Alignment
//! 
//! * `pad_before`/`pad_after` - skip a fixed number of bytes
//! * `align_before`/`align_after` - skip bytes until aligned
//! * `seek_before` - attribute form of calling [`Seek::seek`](crate::io::Seek::seek)
//! * `pad_size_to` - skips to a certain number past the start of this field if that point hasn't
//! already been passed
//! 
//! ```rust
//! # use binread::{BinRead, NullString, io::SeekFrom};
//! 
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(align_before = 4, pad_after = 1, align_after = 4)]
//!     str: NullString,
//! 
//!     #[br(pad_size_to = 0x10)]
//!     test: u64,
//!     
//!     #[br(seek_before = SeekFrom::End(-4))]
//!     end: u32,
//! }
//! ```