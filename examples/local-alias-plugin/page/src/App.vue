<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ipc } from "./lib/ipc";

const version = "v0.6.0";

const authorUrl = "https://sevenc7c.com";
const repoUrl = "https://github.com/sevenc-nanashi/aviutl2-rs";
const treeUrl = `https://github.com/sevenc-nanashi/aviutl2-rs/tree/${version}/examples/local-alias-plugin`;

const pluginVersion = ref<string>("(unknown)");

onMounted(() => {
  ipc.getVersion().then((v) => (pluginVersion.value = v));
});
const alert = window.alert;
</script>

<template>
  <main>
    <h1>プロジェクト内のエイリアス</h1>
    <div class="main-container">
      <div
        style="display: flex; gap: 8px; align-items: center; margin-bottom: 8px"
      >
        Get Aliases:
        <button
          @click="
            ipc
              .getAliases()
              .then((aliases) => alert(JSON.stringify(aliases, null, 2)))
          "
        >
          Click
        </button>
      </div>
    </div>
    <footer>
      <p>
        Rusty Local Alias Plugin - v{{ pluginVersion }}, by
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
          >examples/local-alias-plugin</a
        >
      </p>
    </footer>
  </main>
</template>

<style scoped>
.main-container {
  width: 100%;
}

.author-link {
  color: #48b0d5;
}
</style>
