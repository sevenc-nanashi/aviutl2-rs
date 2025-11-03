<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ipc } from "./lib/ipc.ts";

const version = ref<string>("(unknown)");
const authorUrl = "https://sevenc7c.com";
const repoUrl = "https://github.com/sevenc-nanashi/aviutl2-rs";
const treeUrl = computed(
  () =>
    `https://github.com/sevenc-nanashi/aviutl2-rs/tree/${version.value}/examples/local-alias-plugin`,
);

onMounted(() => {
  ipc.getVersion().then((v) => (version.value = v));
});
const alert = window.alert.bind(window);
</script>

<template>
  <main>
    <div class="main-container">
      <p>A</p>
      <p>B</p>
    </div>
  </main>
</template>

<style scoped>
.main-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-wrap: wrap;
}

.author-link {
  color: #48b0d5;
}
</style>
