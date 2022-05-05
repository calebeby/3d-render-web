import "./style.css";

import * as wasm from "../rust/pkg/rust.js";
await wasm.default();

const app = document.querySelector<HTMLDivElement>("#app")!;

const canvas = document.createElement("canvas");
app.append(canvas);

const ctx = canvas.getContext("2d")!;

interface Polygon {
  points: [x: number, y: number][];
  color: string;
}

const unpackData = (data: Float64Array) => {
  let polygonIndex = 0;
  const polygons: Polygon[] = [];
  while (polygonIndex < data.length) {
    const numPoints = data[polygonIndex];
    const color = data[polygonIndex + 1];

    const points: [x: number, y: number][] = [];

    for (let i = 0; i < numPoints; i++) {
      const x = data[polygonIndex + 2 + i * 2];
      const y = data[polygonIndex + 2 + i * 2 + 1];
      points.push([x, y]);
    }

    const red = (color & 0xff0000) >> 16;
    const green = (color & 0x00ff00) >> 8;
    const blue = color & 0x0000ff;

    polygons.push({
      points,
      color: `rgb(${red}, ${green}, ${blue})`,
    });

    polygonIndex += 2 + numPoints * 2;
  }
  return polygons;
};

const render = (mouseX: number, mouseY: number, mouseDown: boolean) => {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  canvas.width = width;
  canvas.height = height;
  const data = wasm.get_points(ctx, width, height, mouseX, mouseY, mouseDown);

  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, width, height);

  const polygons = unpackData(data);

  for (const polygon of polygons) {
    ctx.fillStyle = polygon.color;
    ctx.beginPath();
    for (const point of polygon.points) {
      ctx.lineTo(point[0] + width / 2, point[1] + height / 2);
    }
    ctx.closePath();
    ctx.fill();
  }
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
