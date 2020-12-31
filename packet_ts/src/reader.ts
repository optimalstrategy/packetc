
export class Reader {
    private pointer: number;
    private view: DataView;

    constructor(data: ArrayBuffer) {
        this.pointer = 0;
        this.view = new DataView(data);
    }

    remaining(): number {
        return this.view.byteLength - this.pointer;
    }

    private advance(by: number) {
        this.pointer += by;
        return this.pointer - by;
    }

    // Reads a single u8, does not do bounds checking.

    read_uint8(): number {
        return this.view.getUint8(this.advance(1));
    }

    // Reads a single u16, does not do bounds checking.

    read_uint16(): number {
        return this.view.getUint16(this.advance(2), true);
    }

    // Reads a single u32, does not do bounds checking.

    read_uint32(): number {
        return this.view.getUint32(this.advance(4), true);
    }

    // Reads a single i8, does not do bounds checking.

    read_int8(): number {
        return this.view.getInt8(this.advance(1));
    }

    // Reads a single i16, does not do bounds checking.

    read_int16(): number {
        return this.view.getInt16(this.advance(2), true);
    }

    // Reads a single i32, does not do bounds checking.
    read_int32(): number {
        return this.view.getInt32(this.advance(4), true);
    }

    // Reads a single f32, does not do bounds checking.
    read_float(): number {
        return this.view.getFloat32(this.advance(4), true);
    }

    // Reads a slice of `len`, does not do bounds checking
    read_slice(len: number): ArrayBufferLike {
        const pos = this.advance(len);
        return this.view.buffer.slice(pos, pos + len);
    }
}