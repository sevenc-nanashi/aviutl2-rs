import { reactive, readonly } from "vue";
import { ipc, type AliasEntry } from "./ipc.ts";

export type State = {
  aliases: AliasEntry[];
  selectedIndex: number | null;
};

const state = reactive<State>({
  aliases: [],
  selectedIndex: null,
});

function setAliases(aliases: AliasEntry[]) {
  state.aliases = Array.isArray(aliases) ? aliases : [];
  // 長さ変化による選択インデックスのはみ出しを補正
  if (state.selectedIndex != null) {
    if (state.selectedIndex < 0 || state.selectedIndex >= state.aliases.length) {
      state.selectedIndex = null;
    }
  }
}

function setSelectedIndex(index: number | null) {
  state.selectedIndex = index;
  ipc.setCurrentAlias(state.aliases[index ?? -1] ?? null);
}

// Rust 側からの push 通知を受け取る
ipc.on("update_aliases", (data) => {
  try {
    setAliases((data as AliasEntry[]) ?? []);
  } catch (e) {
    console.error(e);
  }
});

// 初期化時、必要なら取得（開発時など）
ipc
  .getAliases()
  .then((aliases) => setAliases(aliases))
  .catch(() => void 0);

export function useGlobalStore() {
  return {
    state: readonly(state),
    setAliases,
    saveAliases: (aliases: AliasEntry[]) => ipc.setAliases(aliases),
    setSelectedIndex,
  };
}
