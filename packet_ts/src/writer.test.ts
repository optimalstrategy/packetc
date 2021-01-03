import { Writer } from "./writer";

type BufferType = "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "f32";
function make_buffer(type: BufferType, value: number) {
    let size = 0;
    switch (type) {
        case "u8": size = 1; break;
        case "u16": size = 2; break;
        case "u32": size = 4; break;
        case "i8": size = 1; break;
        case "i16": size = 2; break;
        case "i32": size = 4; break;
        case "f32": size = 4; break;
        default: throw new Error(`??????`);
    }
    const buffer = new ArrayBuffer(size);
    const view = new DataView(buffer);
    switch (type) {
        case "u8": view.setUint8(0, value); break;
        case "u16": view.setUint16(0, value, true); break;
        case "u32": view.setUint32(0, value, true); break;
        case "i8": view.setInt8(0, value); break;
        case "i16": view.setInt16(0, value, true); break;
        case "i32": view.setInt32(0, value, true); break;
        case "f32": view.setFloat32(0, value, true); break;
        default: throw new Error(`??????`);
    }
    return new Uint8Array(buffer);
}

type WriterMethodKey = Exclude<keyof Writer, "write_string">;

const cases: [string, number, Uint8Array][] = [
    ["uint8", 100, make_buffer("u8", 100)],
    ["uint16", 10000, make_buffer("u16", 10000)],
    ["uint32", 1_000_000_000, make_buffer("u32", 1_000_000_000)],
    ["int8", 100, make_buffer("u8", 100)],
    ["int16", 10000, make_buffer("u16", 10000)],
    ["int32", 1_000_000_000, make_buffer("u32", 1_000_000_000)],
    ["float", 10.5, make_buffer("f32", 10.5)],
];
describe("Reader scalar", function () {
    for (const test_case of cases) {
        const [type, value, expected] = test_case;
        it(`write_${type}`, function () {
            const writer = new Writer(expected.byteLength);
            writer[`write_${type}` as WriterMethodKey](value);
            const actual = writer.finish();
            expect(new Uint8Array(actual)).toEqual(expected);
        });
    }

    it(`write_string`, function () {
        const expected = new TextEncoder().encode("testing");
        const writer = new Writer();
        writer.write_string("testing");
        const actual = writer.finish();
        expect(new Uint8Array(actual)).toEqual(expected);
    });
});