import typescript from "rollup-plugin-typescript2";
import pkg from "./package.json";
import { terser } from "rollup-plugin-terser";

export default [
    // UMD, CJS, ESM
    {
        input: "src/index.ts",
        plugins: [
            typescript({
                typescript: require("typescript"),
            }),
            terser({
                output: {
                    comments: false,
                },
            }),
        ],
        output: [
            { exports: "named", file: pkg.main, format: "cjs" },
            { file: pkg.module, format: "es" },
            { name: "packet", file: pkg.browser, format: "umd" },
        ],
    },
];
