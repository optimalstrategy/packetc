Binary serialization schema format

**Why another one?**
None of the other formats exactly fit my needs - I need to pass data to/from JS in a low-bandwidth, low-latency way. This library will be a lot simpler than all of the popular ones:
 * No versioning
 * No namespacing
 * No type definitions
 * No RPC 
 * Only a few basic types
   * You can compose them to form more complex types, but only linearly - no recursive types.

Why use a schema? Schema-less formats like JSON, CBOR, BSON, MessagePack are easy to use, but they are extremely wasteful, and error-prone. It's hard to maintain compatibility between different languages and environments. A schema makes it easy to keep packet parsing in sync, and allows for many optimizations.
I want this library to be fast, produce small packets, and be safe:

* Safe, meaning that a crafted malicious packet won't crash the server, but will only cause an error to be output somewhere - you can quickly drop the connection.
* Small, meaning the packets will serialize into something that only contains the absolutely necessary information, and besides bit-packing and compression, it would be impossible to shrink the packet any further by hand.
* Fast meaning in the best, most common case (the packet is not malicious), parsing packets won't be a bottleneck.

**What will it look like?**
Because the target is Rust and JavaScript, the lowest common denominator determines what the format must look like. In this case JS, because all types in JS can be somehow efficiently represented in Rust. 

Here's some syntax:

Symbols are declared as `identifier: type`, where `type` is any of:
```
- array, in the form `identifier: type[]`
    - output:
        - rs: Vec<type>
        - ts: Array<type>
- struct, in the form `identifier: struct { name0:type0, name1:type1, ..., nameN:typeN }`
    - output:
    - rs: struct `identifier` { name0: type0, name1: type1, ..., nameN: typeN }
    - ts: interface `identifier` { "name0": type0, "name1": type1, ..., "nameN": typeN }
- enum, in the form `identifier: enum { VARIANT_A, VARIANT_B }`
    - output:
        - rs: enum { VARIANT_A = 1 << 0, VARIANT_B = 1 << 1 }
        - ts: enum { VARIANT_A = 1 << 0, VARIANT_B = 1 << 1 }
- string
    - output:
        - rs: String
        - ts: string
- uint8, uint16, uint32, int8, int16, int32
        - output:
        - rs: u8, u16, u32, i8, i16, i32
        - ts: number
- float
    - output:
        - rs: f32
        - ts: number
```

Comments start with #, and are only single-line.

```s
# This is a comment.
# Below is what a fairly complex packet may look like
Flag: enum { A, B }
Position: struct { x: float, y: float }
Value: struct { 
  a: uint32, b: int32, c: uint8, d: uint8
}
ComplexType: struct {
    flag: Flag,
    positions: Position[],
    names: string[],
    values: Value[],
    # Optional types exist, too
    test?: uint8,
}

export ComplexType
```

**Implementation:**
Uses [peg](https://github.com/kevinmehall/rust-peg) for defining parsing.

The general idea is:
1. Load a schema file
2. Run it through the parser, generating an AST
3. Traverse the AST, and execute a simple type checking pass, where all type names are resolved.
4. Traverse the resolved AST, generating the structs/interfaces and the read/write implementations.
5. Write it to a .rs/.ts file.

**Status**

Currently, the compiler is fully functional. It can compile schemas into Rust and TypeScript code, which depends on the corresponding [`packet-rs`](https://github.com/EverCrawl/packet-rs) and [`packet-ts`](https://github.com/EverCrawl/packet-ts) libraries.

**Notes**

Recursive types are not possible. This is because besides arrays,
there is no other form of indirection.
```s
T: struct {
    v: T # Error
}

export T
```

It's currently not possible to have optional array fields, such as:
```s
T: struct {
    opt?: A[] # This is an error
}

export T
```
It's on the to-do list, but not a high priority, because you can always
represent such a scenario with a flag.