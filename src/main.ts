import "./style.css";

import * as wasm from "../rust/pkg/rust.js";
await wasm.default();

const app = document.querySelector<HTMLDivElement>("#app")!;

const canvas = document.createElement("canvas");
app.append(canvas);

const ctx = canvas.getContext("2d")!;

const render = (mouseX: number, mouseY: number, mouseDown: boolean) => {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  canvas.width = width;
  canvas.height = height;
  wasm.render(ctx, width, height, mouseX, mouseY, mouseDown);
};

render(0, 0, false);

const handleMouseEvent = (event: MouseEvent) => {
  const x = event.offsetX;
  const y = event.offsetY;
  render(x, y, event.buttons === 1);
};
canvas.addEventListener("mousedown", handleMouseEvent);
canvas.addEventListener("mouseup", handleMouseEvent);
canvas.addEventListener("mousemove", handleMouseEvent);

window.addEventListener("resize", () => {
  render(0, 0, false);
});
