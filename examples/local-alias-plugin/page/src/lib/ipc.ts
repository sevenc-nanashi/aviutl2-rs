import { getBridge } from "./bridge.ts";

export type AliasEntry = { name: string; alias: string };

const bridge = getBridge();

const onceWaiters = new Map<string, Array<(data: unknown) => void>>();
const subscribers = new Map<string, Array<(data: unknown) => void>>();

bridge.onMessage((type, data) => {
  const onceList = onceWaiters.get(type);
  if (onceList && onceList.length) {
    const callbacks = onceList.splice(0, onceList.length);
    onceWaiters.delete(type);
    for (const cb of callbacks) {
      try {
        cb(data);
      } catch (e) {
        console.error(e);
      }
    }
  }

  const subs = subscribers.get(type);
  if (subs && subs.length) {
    for (const cb of subs) {
      try {
        cb(data);
      } catch (e) {
        console.error(e);
      }
    }
  }
});

function waitFor<T = unknown>(type: string): Promise<T> {
  return new Promise<T>((resolve) => {
    const list = onceWaiters.get(type) ?? [];
    list.push((data) => resolve(data as T));
    onceWaiters.set(type, list);
  });
}

export const ipc = {
  async getVersion(): Promise<string> {
    bridge.send("get_version", {});
    const res = await waitFor<{ version: string }>("version_response");
    return res.version ?? "";
  },

  async getAliases(): Promise<AliasEntry[]> {
    bridge.send("get_aliases", {});
    const res = await waitFor<AliasEntry[]>("aliases_response");
    return Array.isArray(res) ? res : [];
  },

  setAliases(aliases: AliasEntry[]): void {
    bridge.send("set_aliases", aliases);
  },

  on(type: string, cb: (data: unknown) => void): void {
    const list = subscribers.get(type) ?? [];
    list.push(cb);
    subscribers.set(type, list);
  },
};
