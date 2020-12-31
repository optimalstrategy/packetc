
const GROWTH_FACTOR = 2;

export class Writer {
    private pointer: number;
    private arrayView: Uint8Array;
    private view: DataView;

    constructor(capacity?: number) {
        this.pointer = 0;
        const buffer = new ArrayBuffer(capacity ?? 0);
        this.arrayView = new Uint8Array(buffer);
        this.view = new DataView(buffer);
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
    write_slice(value: Uint8Array) {
        this.ensure(value.byteLength);
        const pos = this.advance(value.byteLength);
        this.arrayView.set(value, pos);
    }
    finish(): Uint8Array {
        return this.arrayView.slice(0, this.pointer);
    }
}
