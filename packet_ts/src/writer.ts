
const GROWTH_FACTOR = 2;

/* // returns the byte length of a UTF-8 string, following RFC3629 (at most 4 byte characters)
// source: https://stackoverflow.com/a/23329386/11953579
function byteLength(str: string) {
    let s = str.length;
    for (let i = str.length - 1; i >= 0; i--) {
        const code = str.charCodeAt(i);
        if (code > 0x7f && code <= 0x7ff) s++;
        else if (code > 0x7ff && code <= 0xffff) s += 2;
        if (code >= 0xDC00 && code <= 0xDFFF) i--; //trail surrogate
    }
    return s;
} */

/**
 * A TextEncoder with a temporary, pre-allocated buffer. 
 * If re-used, can offer ~2x the performance of TextEncoder alone.
 */
class StringEncoder {
    private encoder = new TextEncoder();
    private byteLength = 0;
    private buffer = new Uint8Array(1024);
    /**
     * Encodes `value` into a temporary buffer, and returns the number of bytes encoded.
     * @param value 
     */
    encode(value: string): number {
        this.byteLength = this.encoder.encodeInto(value, this.buffer).written!;
        return this.byteLength;
    }

    /**
     * Copy the bytes of the previously encoded string into `dst`, staring at `dstOffset`.
     * @param dst 
     * @param dstOffset 
     */
    getInto(dst: Uint8Array, dstOffset: number) {
        dst.set(this.buffer.slice(0, this.byteLength), dstOffset);
    }
}

export class Writer {
    private pointer: number;
    private arrayView: Uint8Array;
    private view: DataView;
    private encoder: StringEncoder;

    /**
     * Default constructor
     */
    constructor();
    /**
     * Construct with capacity
     * @param capacity 
     */
    constructor(capacity: number);
    /**
     * Construct from an existing buffer
     * @param buffer 
     */
    constructor(buffer: ArrayBuffer);
    constructor(arg0?: number | ArrayBuffer) {
        this.pointer = 0;
        const buffer = arg0 instanceof ArrayBuffer ? arg0 : new ArrayBuffer(arg0 ?? 0);
        this.arrayView = new Uint8Array(buffer);
        this.view = new DataView(buffer);
        this.encoder = new StringEncoder;
    }

    // If needed, resize to fit at least another `additional` bytes
    private ensure(additional: number) {
        if (this.view.byteLength >= this.pointer + additional) {
            return;
        }
        // allocate new buffer
        const newBuffer = new ArrayBuffer(this.view.byteLength + additional * GROWTH_FACTOR);
        // copy old -> new
        const slice = new Uint8Array(newBuffer);
        slice.set(this.arrayView);
        // update slice & view
        this.arrayView = slice;
        this.view = new DataView(newBuffer);
    }

    private advance(by: number) {
        this.pointer += by;
        return this.pointer - by;
    }

    write_uint8(value: number) {
        this.ensure(1);
        this.view.setUint8(this.advance(1), value);
    }

    write_uint16(value: number) {
        this.ensure(2);
        this.view.setUint16(this.advance(2), value, true);
    }

    write_uint32(value: number) {
        this.ensure(4);
        this.view.setUint32(this.advance(4), value, true);
    }

    write_int8(value: number) {
        this.ensure(1);
        this.view.setInt8(this.advance(1), value);
    }

    write_int16(value: number) {
        this.ensure(2);
        this.view.setInt16(this.advance(2), value, true);
    }

    write_int32(value: number) {
        this.ensure(4);
        this.view.setInt32(this.advance(4), value, true);
    }

    write_float(value: number) {
        this.ensure(4);
        this.view.setFloat32(this.advance(4), value, true);
    }

    write_string(value: string) {
        const len = this.encoder.encode(value);
        this.ensure(len);
        this.encoder.getInto(this.arrayView, this.advance(len));
    }

    finish(): ArrayBuffer {
        return this.view.buffer.slice(0, this.pointer);
    }
}
