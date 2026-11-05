import { invoke } from "@tauri-apps/api/core";

export type LogLevel = "trace" | "debug" | "info" | "warn" | "error";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);
}

const logQueue: Array<{ level: string; tag: string; message: string }> = [];
let isFlushing = false;
let isLoggingInternal = false;

async function sendToBackend(level: string, tag: string, message: string): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  logQueue.push({ level, tag, message });
  if (isFlushing) return;

  isFlushing = true;
  while (logQueue.length > 0) {
    const item = logQueue.shift();
    if (!item) break;
    try {
      await invoke("log_frontend", {
        level: item.level,
        tag: item.tag,
        message: item.message,
      });
    } catch {
      // Avoid infinite error loop if Tauri IPC is not ready yet
    }
  }
  isFlushing = false;
}

function formatArg(arg: unknown): string {
  if (arg === null) return "null";
  if (arg === undefined) return "undefined";
  if (arg instanceof Error) {
    return `${arg.name}: ${arg.message}${arg.stack ? `\n${arg.stack}` : ""}`;
  }
  if (typeof arg === "object") {
    try {
      return JSON.stringify(
        arg,
        (key, value) => {
          if (/api_?key|token|password|secret/i.test(key) && typeof value === "string") {
            return "[REDACTED]";
          }
          if (Array.isArray(value) && value.length > 50 && typeof value[0] === "number") {
            return `[Array of ${value.length} bytes]`;
          }
          return value;
        },
        2
      );
    } catch {
      return "[Circular or Unserializable Object]";
    }
  }
  return String(arg);
}

function formatMessage(message: string, ...args: unknown[]): string {
  if (args.length === 0) return message;
  const formattedArgs = args.map(formatArg).join(" ");
  return message ? `${message} ${formattedArgs}` : formattedArgs;
}

export const picoLog = {
  trace(tag: string, message: string, ...args: unknown[]) {
    if (isLoggingInternal) return;
    const formatted = formatMessage(message, ...args);
    sendToBackend("trace", tag, formatted);
  },
  debug(tag: string, message: string, ...args: unknown[]) {
    if (isLoggingInternal) return;
    const formatted = formatMessage(message, ...args);
    sendToBackend("debug", tag, formatted);
  },
  info(tag: string, message: string, ...args: unknown[]) {
    if (isLoggingInternal) return;
    const formatted = formatMessage(message, ...args);
    sendToBackend("info", tag, formatted);
  },
  warn(tag: string, message: string, ...args: unknown[]) {
    if (isLoggingInternal) return;
    const formatted = formatMessage(message, ...args);
    sendToBackend("warn", tag, formatted);
  },
  error(tag: string, message: string, ...args: unknown[]) {
    if (isLoggingInternal) return;
    const formatted = formatMessage(message, ...args);
    sendToBackend("error", tag, formatted);
  },
};

let initialized = false;

export function initFrontendLogger(): void {
  if (initialized || typeof window === "undefined") return;
  initialized = true;

  picoLog.info("frontend", "Frontend logger initialized. Logging to ~/.pico.log");

  const origLog = console.log.bind(console);
  const origInfo = console.info.bind(console);
  const origWarn = console.warn.bind(console);
  const origError = console.error.bind(console);
  const origDebug = console.debug.bind(console);

  console.log = (...args: unknown[]) => {
    origLog(...args);
    isLoggingInternal = true;
    try {
      picoLog.info("console", "", ...args);
    } finally {
      isLoggingInternal = false;
    }
  };

  console.info = (...args: unknown[]) => {
    origInfo(...args);
    isLoggingInternal = true;
    try {
      picoLog.info("console", "", ...args);
    } finally {
      isLoggingInternal = false;
    }
  };

  console.warn = (...args: unknown[]) => {
    origWarn(...args);
    isLoggingInternal = true;
    try {
      picoLog.warn("console", "", ...args);
    } finally {
      isLoggingInternal = false;
    }
  };

  console.error = (...args: unknown[]) => {
    origError(...args);
    isLoggingInternal = true;
    try {
      picoLog.error("console", "", ...args);
    } finally {
      isLoggingInternal = false;
    }
  };

  console.debug = (...args: unknown[]) => {
    origDebug(...args);
    isLoggingInternal = true;
    try {
      picoLog.debug("console", "", ...args);
    } finally {
      isLoggingInternal = false;
    }
  };

  window.addEventListener("error", (event) => {
    const details = event.error?.stack || `${event.message} at ${event.filename}:${event.lineno}:${event.colno}`;
    picoLog.error("window::uncaught_error", details);
  });

  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason;
    const details = reason instanceof Error ? `${reason.message}\n${reason.stack}` : formatArg(reason);
    picoLog.error("window::unhandledrejection", details);
  });
}
