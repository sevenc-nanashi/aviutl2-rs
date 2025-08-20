import { inject, type Plugin } from "vue";
import * as base64 from "base64-js";

export type RenderData = {
  version: string;
  width: number;
  height: number;
  msPerFrame: number[];
  numFrames: number;
  totalMs: number;
  fps: number;
  startTime: string;
  endTime: string;
};

function generateDummyData(): RenderData {
  const numFrames = 60 * 5;
  const msPerFrame = [];
  let currentFrameBase = 100;
  for (let i = 0; i < numFrames; i++) {
    if (Math.random() < 0.1) {
      currentFrameBase = randBetween(50, 150);
    }
    msPerFrame.push(currentFrameBase + randBetween(-10, 10));
  }
  const totalMs = msPerFrame.reduce((a, b) => a + b, 0);
  return {
    version: "0.0.0-dummy",
    width: 1920,
    height: 1080,
    msPerFrame,
    numFrames,
    totalMs,
    fps: 60,
    startTime: new Date().toISOString(),
    endTime: new Date(Date.now() + totalMs).toISOString(),
  };
}
function randBetween(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

export const renderDataPlugin: Plugin = {
  install(app) {
    const dataElement = document.getElementById("data");
    if (!dataElement) {
      throw new Error("Data element not found");
    }
    const encodedData = dataElement.getAttribute("data-render-data");
    if (!encodedData) {
      throw new Error("Render data not found");
    }
    if (encodedData === "!PLACEHOLDER!") {
      const dummyData = generateDummyData();
      app.provide("renderData", dummyData);
      return;
    }
    const parsedData = JSON.parse(
      new TextDecoder().decode(base64.toByteArray(encodedData)),
    );
    if (!parsedData || typeof parsedData !== "object") {
      throw new Error("Invalid data format");
    }
    app.provide("renderData", parsedData as RenderData);
  },
};

export function useRenderData() {
  const renderData = inject<RenderData>("renderData");
  if (!renderData) {
    throw new Error("Render data not provided");
  }
  return renderData;
}
