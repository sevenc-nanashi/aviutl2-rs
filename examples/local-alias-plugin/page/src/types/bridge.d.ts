declare global {
  interface Window {
    bridge?: {
      // Send arbitrary payload to the plugin (JSON-serializable recommended)
      send: (data: unknown) => void;
      // Subscribe to plugin -> JS messages
      onMessage: (cb: (msg: unknown) => void) => void;
      // Low-level emit used by plugin runtime
      _emit?: (msg: unknown) => void;
    };
  }
}
