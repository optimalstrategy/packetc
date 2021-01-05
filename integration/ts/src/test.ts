import { Reader, Writer } from "packet";
import { read, write, ComplexType, Flag } from "./generated";

describe("generated", function () {
    it("works", function () {
        let data: ComplexType = {
            names: ["first", "second"],
            positions: [{ x: 0.0, y: 1.0 }],
            flag: Flag.B,
            values: [{
                a: 0,
                b: 1,
                c: 30,
                d: 100,
            }],
        };
        let bin = new Uint8Array([
            2, 0, 0, 0, // names.len()
            5, 0, 0, 0, // names[0].len()
            102, 105, 114, 115, 116, // names[0]
            6, 0, 0, 0, // names[1].len()
            115, 101, 99, 111, 110, 100, // names[1]
            1, 0, 0, 0, // positions.len()
            0, 0, 0, 0, // positions[0].x
            0, 0, 128, 63, // positions[0].y
            2,  // flag
            1, 0, 0, 0, // values.len()
            0, 0, 0, 0, // values[0].a
            1, 0, 0, 0,   // values[0].b
            30,  // values[0].c
            100, // values[0].d
        ]).buffer;
        {
            let writer = new Writer();
            write(writer, data);
            expect(writer.finish()).toEqual(bin);
        }

        {
            let reader = new Reader(bin);
            // @ts-ignore
            let actual: ComplexType = {};
            read(reader, actual);
            expect(actual).toEqual(data);
        }

    });
});