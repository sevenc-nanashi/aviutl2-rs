<script setup lang="ts">
import Statistics from "./components/Statistics.vue";
import FrameChart from "./components/FrameChart.vue";
import { useRenderData } from "./useRenderData";
const { version } = useRenderData();

const authorUrl = "https://sevenc7c.com";
const repoUrl = "https://github.com/sevenc-nanashi/aviutl2-rs";
const treeUrl = `https://github.com/sevenc-nanashi/aviutl2-rs/tree/${version}/examples/statistics-output`;
</script>

<template>
  <main>
    <h1>レンダリング統計</h1>
    <Statistics />
    <div class="chart-container">
      <FrameChart />
    </div>
    <footer>
      <p>
        Rusty Statistics Plugin - v{{ version }}, by
        <a
          class="author-link"
          :href="authorUrl"
          target="_blank"
          rel="noopener noreferrer"
          >Nanashi.</a
        >
        /
        <a :href="repoUrl" target="_blank" rel="noopener noreferrer"
          >sevenc-nanashi/aviutl2-rs</a
        >,
        <a :href="treeUrl" target="_blank" rel="noopener noreferrer"
          >examples/statistics-output</a
        >
      </p>
    </footer>
  </main>
</template>

<style scoped>
.chart-container {
  width: 100%;
  height: 400px;
}

.author-link {
  color: #48b0d5;
}
</style>
