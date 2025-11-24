type Bridge = {
  send: (type: string, data: unknown) => void;
  onMessage: (cb: (type: string, data: unknown) => void) => void;
  _emit: (msg: { type: string; data: unknown }) => void;
};

const windowPatched = window as unknown as Window & {
  bridge?: Bridge;
  ipc: { postMessage: (msg: string) => void };
};

if (!windowPatched.bridge) {
  const listeners: Array<(type: string, data: unknown) => void> = [];
  windowPatched.bridge = {
    send: (name, data) => {
      try {
        windowPatched.ipc.postMessage(
          JSON.stringify({ type: name, data: data }),
        );
      } catch (e) {
        console.error(e);
      }
    },
    onMessage: (cb) => {
      if (typeof cb === "function") listeners.push(cb);
    },
    _emit: (msg) => {
      console.debug("[bridge] IPC message received", msg);
      for (const cb of listeners) {
        try {
          cb(msg.type, msg.data);
        } catch (e) {
          console.error(e);
        }
      }
    },
  };
  console.debug("[bridge] IPC bridge initialized");
}

export function getBridge(): Bridge {
  const b = windowPatched.bridge;
  if (!b) {
    throw new Error("IPC bridge (window.bridge) not ready");
  }
  return b;
}
