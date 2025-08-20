<script setup lang="ts">
import { useRenderData } from "../useRenderData";
import { Line } from "vue-chartjs";
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  LineElement,
  PointElement,
  CategoryScale,
  LinearScale,
} from "chart.js";

ChartJS.register(
  Title,
  Tooltip,
  Legend,
  LineElement,
  PointElement,
  CategoryScale,
  LinearScale,
);

const { msPerFrame } = useRenderData();

const chartData = {
  labels: msPerFrame.map((_, i) => i + 1),
  datasets: [
    {
      label: "描画にかかった時間 (ms)",
      data: msPerFrame,
      borderColor: "#42b883",
      backgroundColor: "#42b883",
    },
  ],
};

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
};
</script>

<template>
  <Line :data="chartData" :options="chartOptions" />
</template>
