import "./style.css";

import * as wasm from "../rust/pkg/rust.js";
await wasm.default();

const app = document.querySelector<HTMLDivElement>("#app")!;

const canvas = document.createElement("canvas");
const width = 600;
const height = 600;
canvas.width = width;
canvas.height = height;
const createSlider = () => {
  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = "-10";
  slider.max = "10";
  slider.step = "0.01";
  return slider;
};
const xSlider = createSlider();
const ySlider = createSlider();
const zSlider = createSlider();
xSlider.value = "-10";
app.append(canvas, xSlider, ySlider, zSlider);

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

const render = () => {
  const cameraXOffset = xSlider.valueAsNumber;
  const cameraYOffset = ySlider.valueAsNumber;
  const cameraZOffset = zSlider.valueAsNumber;
  const data = wasm.get_points(
    width,
    height,
    cameraXOffset,
    cameraYOffset,
    cameraZOffset,
  );

  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, width, height);

  const polygons = unpackData(data);

  for (const polygon of polygons) {
    ctx.fillStyle = polygon.color;
    ctx.beginPath();
    for (const point of polygon.points) {
      ctx.lineTo(point[0] + canvas.width / 2, point[1] + canvas.width / 2);
    }
    ctx.closePath();
    ctx.fill();
  }
};

render();

xSlider.addEventListener("input", render);
ySlider.addEventListener("input", render);
zSlider.addEventListener("input", render);
