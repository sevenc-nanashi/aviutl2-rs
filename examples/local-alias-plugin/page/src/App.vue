<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ipc, type AliasEntry } from "./lib/ipc.ts";
import { useGlobalStore } from "./lib/store.ts";

const version = ref<string>("(unknown)");
const authorUrl = "https://sevenc7c.com";
const repoUrl = "https://github.com/sevenc-nanashi/aviutl2-rs";
const treeUrl = computed(
  () =>
    `https://github.com/sevenc-nanashi/aviutl2-rs/tree/${version.value}/examples/local-alias-plugin`,
);

const store = useGlobalStore();
const aliases = computed<readonly AliasEntry[]>(() => store.state.aliases);
onMounted(() => {
  ipc.getVersion().then((v) => (version.value = v));
});
</script>

<template>
  <main>
    <button>エイリアスを登録</button>
    <div class="main-container">
      <div v-for="alias in aliases" :key="alias.name">
        <h3>{{ alias.name }}</h3>
      </div>
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
