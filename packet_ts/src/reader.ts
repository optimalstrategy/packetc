
export class Reader {
    private pointer: number;
    private arrayView: Uint8Array;
    private view: DataView;
    private decoder: TextDecoder;

    constructor(data: ArrayBuffer) {
        this.pointer = 0;
        this.arrayView = new Uint8Array(data);
        this.view = new DataView(data);
        this.decoder = new TextDecoder;
    }

    private advance(by: number) {
        this.pointer += by;
        if (this.pointer > this.view.byteLength) {
            throw new Error("Out of bounds");
        }
        return this.pointer - by;
    }

    // Reads a single u8, may throw in case of out of bounds read.
    read_uint8(): number {
        return this.view.getUint8(this.advance(1));
    }

    // Reads a single u16, may throw in case of out of bounds read.
    read_uint16(): number {
        return this.view.getUint16(this.advance(2), true);
    }

    // Reads a single u32, may throw in case of out of bounds read.
    read_uint32(): number {
        return this.view.getUint32(this.advance(4), true);
    }

    // Reads a single i8, may throw in case of out of bounds read.
    read_int8(): number {
        return this.view.getInt8(this.advance(1));
    }

    // Reads a single i16, may throw in case of out of bounds read.
    read_int16(): number {
        return this.view.getInt16(this.advance(2), true);
    }

    // Reads a single i32, may throw in case of out of bounds read.
    read_int32(): number {
        return this.view.getInt32(this.advance(4), true);
    }

    // Reads a single f32, may throw in case of out of bounds read.
    read_float(): number {
        return this.view.getFloat32(this.advance(4), true);
    }

    // Reads a slice of `len`, may throw in case of out of bounds read.
    read_string(len: number): string {
        const pos = this.advance(len);
        return this.decoder.decode(this.arrayView.slice(pos, pos + len));
    }
}