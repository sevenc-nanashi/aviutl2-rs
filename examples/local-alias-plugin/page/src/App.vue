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
const selectedIndex = computed(() => store.state.selectedIndex);
const showInfo = ref(false);
onMounted(() => {
  ipc.getVersion().then((v) => (version.value = v));
});

const addAlias = () => {
  ipc.addAlias();
};
const setCurrentAlias = (alias: AliasEntry, index: number) => {
  ipc.setCurrentAlias(alias);
  store.setSelectedIndex(index);
};

const renameAlias = (index: number) => {
  const alias = aliases.value[index]!;
  const newName = window.prompt("新しいエイリアス名", alias.name)?.trim();
  if (!newName || newName === alias.name) return;
  const updated = aliases.value.map((a) =>
    a.name === alias.name ? { ...a, name: newName } : a,
  );
  store.setAliases(updated);
  store.saveAliases(updated);
  // 選択はインデックス基準なので変更不要
};

const deleteAlias = (index: number) => {
  const alias = aliases.value[index]!;
  if (!window.confirm(`エイリアス "${alias.name}" を削除しますか？`)) return;
  const updated = aliases.value.filter((_, i) => i !== index);
  store.setAliases(updated);
  store.saveAliases(updated);
  if (selectedIndex.value != null) {
    if (selectedIndex.value === index) store.setSelectedIndex(null);
    else if (selectedIndex.value > index)
      store.setSelectedIndex(selectedIndex.value - 1);
  }
};

const moveAlias = (index: number, dir: -1 | 1) => {
  const newIdx = index + dir;
  if (newIdx < 0 || newIdx >= aliases.value.length) return;
  const updated = aliases.value.slice();
  const [item] = (updated as AliasEntry[]).splice(index, 1);
  (updated as AliasEntry[]).splice(newIdx, 0, item);
  store.setAliases(updated as AliasEntry[]);
  store.saveAliases(updated as AliasEntry[]);
  if (selectedIndex.value === index) store.setSelectedIndex(newIdx);
};
</script>

<template>
  <main class="page-root">
    <header class="toolbar">
      <div class="title">Local Alias Plugin</div>
      <div class="spacer" />
      <button class="btn primary" @click="addAlias">エイリアスを登録</button>
      <button class="btn" @click="showInfo = true">情報</button>
    </header>

    <section class="grid">
      <div
        v-for="(alias, i) in aliases"
        :key="i"
        class="card"
        :class="{ selected: i === selectedIndex }"
      >
        <button
          class="alias-name one-line"
          @click="setCurrentAlias(alias, i)"
          :title="'選択: ' + alias.name"
        >
          {{ alias.name }}
        </button>
        <div class="card-actions row">
          <button
            class="icon-btn"
            title="名前変更"
            aria-label="名前変更"
            @click="renameAlias(i)"
          >
            ✎
          </button>
          <button
            class="icon-btn warn"
            title="削除"
            aria-label="削除"
            @click="deleteAlias(i)"
          >
            🗑
          </button>
          <button
            class="icon-btn"
            title="上へ"
            aria-label="上へ"
            @click="moveAlias(i, -1)"
            :disabled="i === 0"
          >
            ▲
          </button>
          <button
            class="icon-btn"
            title="下へ"
            aria-label="下へ"
            @click="moveAlias(i, 1)"
            :disabled="i === aliases.length - 1"
          >
            ▼
          </button>
        </div>
      </div>
    </section>

    <div v-if="showInfo" class="modal-backdrop" @click.self="showInfo = false">
      <div class="modal">
        <h2>Local Alias Plugin</h2>
        <p>バージョン: {{ version }}</p>
        <ul class="links">
          <li>
            <a :href="authorUrl" target="_blank" rel="noreferrer">Author</a>
          </li>
          <li>
            <a :href="repoUrl" target="_blank" rel="noreferrer">Repository</a>
          </li>
          <li>
            <a :href="treeUrl" target="_blank" rel="noreferrer">This Example</a>
          </li>
        </ul>
        <div class="modal-actions">
          <button class="btn" @click="showInfo = false">閉じる</button>
        </div>
      </div>
    </div>
  </main>
</template>

<style lang="scss" scoped>
// colors
$bg: #0f1115;
$fg: #e6e8ed;
$panel: #141821;
$muted: #2a2f3a;
$muted-2: #1b2030;
$hover: #23293b;
$primary: #2a5bd7;
$primary-hover: #2e65f0;
$warn: #3a1b1b;
$warn-border: #4b2626;
$warn-hover: #4a2222;

.page-root {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: $bg;
  color: $fg;
  font-family:
    ui-sans-serif,
    system-ui,
    -apple-system,
    Segoe UI,
    Roboto,
    Ubuntu,
    Cantarell,
    Noto Sans,
    Helvetica Neue,
    Arial,
    "Apple Color Emoji",
    "Segoe UI Emoji";
}

// compact header
.toolbar {
  display: flex;
  align-items: center;
  gap: 6px; // was 8px
  padding: 6px 10px; // was 10px 12px
  border-bottom: 1px solid $muted;
  background: #12151c;

  .title {
    font-weight: 600;
    font-size: 0.95rem;
  }
  .spacer {
    flex: 1 1 auto;
  }
  // smaller buttons only in header
  .btn {
    padding: 4px 8px; // was 6px 10px
    font-size: 0.92rem;
  }
}

// buttons
.btn,
.icon-btn {
  background: $muted-2;
  color: $fg;
  border: 1px solid $muted;
  border-radius: 6px;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: $hover;
  }
  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

.btn {
  padding: 6px 10px;
  &.primary {
    background: $primary;
    border-color: $primary;
    &:hover {
      background: $primary-hover;
    }
  }
  &.warn {
    background: $warn;
    border-color: $warn-border;
    &:hover {
      background: $warn-hover;
    }
  }
}

.grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
}

.card {
  border-radius: 10px;
  background: $panel;
  overflow: hidden;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  width: 100%;

  &.selected {
    outline: 2px solid $primary;
  }
}

.alias-name {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  font-weight: 600;
  padding: 10px 10px;
  border-radius: 10px;
  border: none;
  background: transparent;
  color: $fg;
  text-align: left;
  cursor: pointer;
  flex: 1 1 auto;

  &:hover {
    background: $muted-2;
  }
}

.card-actions {
  padding: 0 8px;
}
.row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-btn {
  padding: 6px 8px;
  line-height: 1;
  min-width: 34px;

  &.warn {
    background: $warn;
    border-color: $warn-border;
    &:hover {
      background: $warn-hover;
    }
  }
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: grid;
  place-items: center;
}
.modal {
  background: $panel;
  border: 1px solid $muted;
  border-radius: 10px;
  min-width: 320px;
  max-width: 560px;
  padding: 16px;
}
.links {
  margin-top: 8px;
  margin-left: 16px;
  a {
    color: #48b0d5;
  }
}
.modal-actions {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
