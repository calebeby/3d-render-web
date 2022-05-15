import "./style.css";

import * as wasm from "../rust/pkg/rust.js";
await wasm.default();

wasm.start();
