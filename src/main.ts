import "./style.css";

import * as wasm from "../rust/pkg/twisty_puzzles.js";
await wasm.default();

wasm.start();
