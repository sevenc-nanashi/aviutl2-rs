type IncomingMessage<T = unknown> = { type: string; data: T } | unknown;

function getBridge(): Bridge | undefined {
  if (typeof window === "undefined") return undefined;
  return (window as unknown as { bridge?: Bridge }).bridge;
}

async function waitForBridge(timeoutMs = 2000): Promise<Bridge> {
  const existing = getBridge();
  if (existing) return existing;
  const start = Date.now();
  return await new Promise<Bridge>((resolve, reject) => {
    const t = setInterval(() => {
      const b = getBridge();
      if (b) {
        clearInterval(t);
        resolve(b);
      } else if (Date.now() - start > timeoutMs) {
        clearInterval(t);
        reject(new Error("IPC bridge (window.Plugin) not ready"));
      }
    }, 20);
  });
}

type Pending = {
  match: (msg: IncomingMessage) => boolean;
  resolve: (v: unknown) => void;
  reject: (e: unknown) => void;
  timeoutId?: unknown;
};

const userHandlers: Array<(msg: IncomingMessage) => void> = [];
const pendings: Pending[] = [];
let subscribed = false;

function ensureSubscribed() {
  if (subscribed) return;
  const b = getBridge();
  if (!b?.onMessage) return;
  b.onMessage((msg: IncomingMessage) => {
    // Resolve one-shot waiters first
    for (let i = 0; i < pendings.length; i++) {
      const p = pendings[i];
      try {
        if (p.match(msg)) {
          pendings.splice(i, 1);
          if (p.timeoutId) clearTimeout(p.timeoutId);
          p.resolve(msg);
          break;
        }
      } catch (e) {
        console.error(e);
      }
    }
    // Broadcast to user handlers
    for (const cb of userHandlers) {
      try {
        cb(msg);
      } catch (e) {
        console.error(e);
      }
    }
  });
  subscribed = true;
}

function defaultResponseType(forType: string) {
  return forType.startsWith("get_")
    ? `${forType.slice(4)}_response`
    : `${forType}_response`;
}

function send(payload: string | { type: string; data?: unknown }) {
  const b = getBridge();
  if (!b?.send) {
    console.warn("IPC bridge not ready: dropping message", payload);
    return;
  }
  if (typeof payload === "string") {
    b.send(payload, null);
    return;
  }
  if (payload && typeof payload === "object" && "type" in payload) {
    const { type, data } = payload as { type: string; data?: unknown };
    b.send(type, data ?? null);
    return;
  }
  console.warn("Invalid IPC payload; expected string or { type, data? }");
}

async function invoke<T = unknown>(
  type: string,
  data?: unknown,
  opts?: {
    responseType?: string;
    timeoutMs?: number;
    transform?: (msg: IncomingMessage) => T;
  },
): Promise<T> {
  const b = await waitForBridge();
  ensureSubscribed();
  const expect = opts?.responseType ?? defaultResponseType(type);
  const timeoutMs = opts?.timeoutMs ?? 5000;
  return await new Promise<T>((resolve, reject) => {
    const pending: Pending = {
      match: (msg) => !!msg && typeof msg === "object" && msg.type === expect,
      resolve: (msg) => {
        try {
          if (opts?.transform) resolve(opts.transform(msg));
          else resolve((msg as { data?: unknown }).data as T);
        } catch (e) {
          reject(e);
        }
      },
      reject,
    };
    pending.timeoutId = setTimeout(() => {
      const idx = pendings.indexOf(pending);
      if (idx >= 0) pendings.splice(idx, 1);
      reject(new Error(`IPC timeout awaiting: ${expect}`));
    }, timeoutMs);
    pendings.push(pending);
    try {
      b.send(type, data ?? null);
    } catch (e) {
      reject(e);
    }
  });
}

function on(cb: (msg: IncomingMessage) => void): () => void {
  ensureSubscribed();
  userHandlers.push(cb);
  return () => {
    const i = userHandlers.indexOf(cb);
    if (i >= 0) userHandlers.splice(i, 1);
  };
}

// Specific helpers used by the UI
async function getVersion(): Promise<string> {
  return await invoke<string>("get_version", null, {
    responseType: "version_response",
    transform: (msg) =>
      (msg?.data?.version ?? msg?.version ?? String(msg)) as string,
  });
}

type AliasEntry = { name: string; alias: string };

async function getAliases(): Promise<AliasEntry[]> {
  return await invoke<AliasEntry[]>("get_aliases", null, {
    responseType: "aliases_response",
    transform: (msg) => (msg?.data as AliasEntry[]) ?? [],
  });
}

function setAliases(entries: AliasEntry[]) {
  send({ type: "set_aliases", data: entries });
}

export const ipc = {
  send,
  on,
  getVersion,
  getAliases,
  setAliases,
};

export type { IncomingMessage };
