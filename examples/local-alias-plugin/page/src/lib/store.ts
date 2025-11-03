import { reactive, readonly } from "vue";
import { ipc, type AliasEntry } from "./ipc.ts";

export type State = {
  aliases: AliasEntry[];
};

const state = reactive<State>({
  aliases: [],
});

function setAliases(aliases: AliasEntry[]) {
  state.aliases = Array.isArray(aliases) ? aliases : [];
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
  };
}
