binary serialization schema format

**Why?**
I would like to use something like flatbuffers - but you can't even verify incoming buffers with the Rust version. Formats like JSON, CBOR, BSON, MessagePack are easy to use, but they are extremely wasteful. Not only in raw size, but also speed and overall efficiency, ergonomics, because all work is done at runtime- this is bad for performance, memory usage, makes development difficult due to the lack of a schema, which implies the (de)serialization has to be manually crafted and tested, which promotes "dumb" bugs caused by inattention and slows down development.

* Safe meaning that a crafted malicious packet won't crash the server, but will only cause the connection to be dropped, with no additional harm.
* Small meaning the packets will serialize into something that only contains the absolutely necessary information, and besides bit-packing and compression, it would be impossible to shrink the packet any further.
* Fast meaning in the best, most common case (the packet is not malicious), parsing packets won't be a bottleneck.

**What will it look like?**
Because the target is Rust and JavaScript, the lowest common denominator determines what sort of data we can serialize - in this case that's JS, because all types in JS can be somehow efficiently represented in Rust. 

Syntax:

Comments are lines starting with '#'

Symbols are declared as `identifier: type;`, where `type` is any of:
- uint8, uint16, uint32
    rs: u8, u16, u32
    js: number
- int8, int16, int32
    rs: i8, i16, i32
    js: number
- float
    rs: f32
    js: number
- string
    rs: String
    js: string
- flag, in the form `identifier: { VARIANT_A, VARIANT_B }`
    rs: enum { VARIANT_A = 1 << 0, VARIANT_B = 1 << 1 }
    js: enum { VARIANT_A = 1 << 0, VARIANT_B = 1 << 1 }
- array, in the form `identifier: type[]`, with nesting: `identifier: type[][]`
    rs: Vec<type>
    js: Array<type>
- tuple, in the form `identifier: (name0:type0, name1:type1, ..., nameN:typeN)`
    rs: struct { name0: type0, name1: type1, ..., nameN: typeN }
    js: { "name0": type0, "name1": type1, ..., "nameN": typeN }

```
u8: uint8
u16: uint16
u32: uint32
i8: int8
i16: int16
i32: int32
f32: float
u8_array: uint8[]
array_of_u8_arrays: uint8[][]
str: string
str_array: string[]
tuple: (
    x: float, 
    y: float, 
    name: string
)
tuple_array: (
    x: float, 
    y: float, 
    name: string
)[]
flag: {
    VARIANT_A, 
    VARIANT_B
}
flag_array: {
    VARIANT_A, 
    VARIANT_B
}[]
complex_type: (
    flag: { A, B },
    positions: (x: float, y: float)[],
    names: string[],
    values: (
        a: u32,
        b: i32,
        c: u8,
        d: u8
    )[]
)
```

Implementation:
Use PEG for parsing, construct AST, then
1. Rust: struct definition + parse/write impl >>> <schema_filename>.rs
2. TS: interface + parser/write impl >>> <schema_filename>.ts