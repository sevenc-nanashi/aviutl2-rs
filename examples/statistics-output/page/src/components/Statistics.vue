<script setup lang="ts">
import { useRenderData } from "../useRenderData";

const renderData = useRenderData();

const formatMs = (ms: number) => `${ms.toFixed(2)} ms`;
const formatDateTime = (date: string) => new Date(date).toLocaleString();

const stdDev = (arr: number[]) => {
  const mean = arr.reduce((acc, val) => acc + val, 0) / arr.length;
  return Math.sqrt(
    arr.reduce((acc, val) => acc + Math.pow(val - mean, 2), 0) / arr.length,
  );
};

const contents = [
  {
    フレーム数: renderData.numFrames,
    動画時間: formatMs((renderData.numFrames / renderData.fps) * 1000),
    動画のFPS: renderData.fps.toFixed(2),
    幅: renderData.width,
    高さ: renderData.height,
  },
  {
    描画時間: formatMs(renderData.totalMs),
    開始時間: formatDateTime(renderData.startTime),
    終了時間: formatDateTime(renderData.endTime),
    描画のFPS: (renderData.numFrames / (renderData.totalMs / 1000)).toFixed(2),
  },
  {
    最小時間: formatMs(Math.min(...renderData.msPerFrame)),
    平均時間: formatMs(renderData.totalMs / renderData.numFrames),
    最大時間: formatMs(Math.max(...renderData.msPerFrame)),
    標準偏差: formatMs(stdDev(renderData.msPerFrame)),
    "動画時間/描画時間比": (
      renderData.numFrames /
      renderData.fps /
      (renderData.totalMs / 1000)
    ).toFixed(2),
  },
];
</script>

<template>
  <section class="statistics">
    <table v-for="(column, index) in contents" :key="index">
      <tbody>
        <tr v-for="(value, key) in column" :key="key">
          <th>{{ key }}</th>
          <td>{{ value }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
th {
  text-align: left;
  padding-right: 1rem;
}

td {
  text-align: right;
  padding-left: 1rem;
}

.statistics {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  justify-content: center;
  align-items: flex-start;
}
</style>
