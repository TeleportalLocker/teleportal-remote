import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type SessionState =
  | { kind: "idle" }
  | { kind: "connecting" }
  | { kind: "hosting"; code: string; sessionId: string }
  | {
      kind: "inSession";
      role: string;
      sessionId: string;
      code?: string;
      peerConnected: boolean;
    }
  | { kind: "error"; message: string };

export type ClientConfig = {
  relayUrl: string;
};

type VideoFramePayload = {
  frameId: number;
  width: number;
  height: number;
  timestampMs: number;
  rgbaBase64: string;
};

type RemoteCursorPayload = {
  cursorId: string;
  x: number;
  y: number;
  timestampMs: number;
};

type Screen = "home" | "join" | "session";

type AppModel = {
  screen: Screen;
  relayUrl: string;
  version: string;
  state: SessionState;
  joinCode: string;
  error: string | null;
  busy: boolean;
  updateStatus: string | null;
};

const root = document.querySelector<HTMLDivElement>("#app");
if (!root) {
  throw new Error("#app missing");
}

const model: AppModel = {
  screen: "home",
  relayUrl: "wss://relay.teleportal.fr/ws",
  version: "",
  state: { kind: "idle" },
  joinCode: "",
  error: null,
  busy: false,
  updateStatus: null,
};

let lastFrame: VideoFramePayload | null = null;
let painting = false;
let lastMoveSentMs = 0;
let keyListenerAttached = false;
let lastRemoteCursor: RemoteCursorPayload | null = null;

const MOVE_THROTTLE_MS = 33; // ~30 Hz (contrôle) ; curseur throttlé ~20 Hz côté Rust

type ControlMessage =
  | { type: "mouse_move"; x: number; y: number; timestamp_ms: number }
  | {
      type: "mouse_button";
      button: "left" | "right" | "middle";
      pressed: boolean;
      x: number;
      y: number;
      timestamp_ms: number;
    }
  | {
      type: "mouse_scroll";
      dx: number;
      dy: number;
      x: number;
      y: number;
      timestamp_ms: number;
    }
  | {
      type: "key_event";
      key: string;
      pressed: boolean;
      modifiers: {
        shift: boolean;
        ctrl: boolean;
        alt: boolean;
        meta: boolean;
      };
      timestamp_ms: number;
    };

function nowMs(): number {
  return Date.now();
}

function normCoords(canvas: HTMLCanvasElement, clientX: number, clientY: number): {
  x: number;
  y: number;
} {
  const rect = canvas.getBoundingClientRect();
  const x = rect.width > 0 ? (clientX - rect.left) / rect.width : 0;
  const y = rect.height > 0 ? (clientY - rect.top) / rect.height : 0;
  return {
    x: Math.min(1, Math.max(0, x)),
    y: Math.min(1, Math.max(0, y)),
  };
}

function mapButton(button: number): "left" | "right" | "middle" | null {
  if (button === 0) return "left";
  if (button === 1) return "middle";
  if (button === 2) return "right";
  return null;
}

async function sendControl(message: ControlMessage): Promise<void> {
  if (!isGuestSession(model.state)) return;
  try {
    await invoke("send_control", { message });
  } catch {
    // Ignore transient send errors (session ending, etc.)
  }
}

async function sendCursorPos(x: number, y: number): Promise<void> {
  if (!isGuestSession(model.state)) return;
  try {
    await invoke("send_cursor_pos", { x, y });
  } catch {
    // Ignore transient send errors
  }
}

function updateRemoteCursorMarker(pose: RemoteCursorPayload | null): void {
  const el = document.getElementById("remote-cursor");
  if (!el) return;
  if (!pose) {
    el.hidden = true;
    return;
  }
  el.hidden = false;
  el.style.left = `${pose.x * 100}%`;
  el.style.top = `${pose.y * 100}%`;
}

function clearRemoteCursor(): void {
  lastRemoteCursor = null;
  updateRemoteCursorMarker(null);
}

function remoteCursorMarkup(): string {
  return `<div id="remote-cursor" class="remote-cursor" hidden aria-hidden="true"></div>`;
}

function bindGuestControls(canvas: HTMLCanvasElement): void {
  canvas.tabIndex = 0;
  canvas.style.outline = "none";
  canvas.focus();

  canvas.addEventListener("contextmenu", (e) => e.preventDefault());

  canvas.addEventListener("pointermove", (e) => {
    const { x, y } = normCoords(canvas, e.clientX, e.clientY);
    void sendCursorPos(x, y);
    const t = nowMs();
    if (t - lastMoveSentMs < MOVE_THROTTLE_MS) return;
    lastMoveSentMs = t;
    void sendControl({ type: "mouse_move", x, y, timestamp_ms: t });
  });

  canvas.addEventListener("pointerdown", (e) => {
    canvas.setPointerCapture(e.pointerId);
    const button = mapButton(e.button);
    if (!button) return;
    const { x, y } = normCoords(canvas, e.clientX, e.clientY);
    void sendControl({
      type: "mouse_button",
      button,
      pressed: true,
      x,
      y,
      timestamp_ms: nowMs(),
    });
  });

  canvas.addEventListener("pointerup", (e) => {
    const button = mapButton(e.button);
    if (!button) return;
    const { x, y } = normCoords(canvas, e.clientX, e.clientY);
    void sendControl({
      type: "mouse_button",
      button,
      pressed: false,
      x,
      y,
      timestamp_ms: nowMs(),
    });
  });

  canvas.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      const { x, y } = normCoords(canvas, e.clientX, e.clientY);
      const dy = e.deltaY === 0 ? 0 : e.deltaY > 0 ? -1 : 1;
      const dx = e.deltaX === 0 ? 0 : e.deltaX > 0 ? 1 : -1;
      void sendControl({
        type: "mouse_scroll",
        dx,
        dy,
        x,
        y,
        timestamp_ms: nowMs(),
      });
    },
    { passive: false },
  );
}

function onKeyEvent(e: KeyboardEvent, pressed: boolean): void {
  if (!isGuestSession(model.state) || model.screen !== "session") return;
  if (e.repeat && pressed) return;
  // Ne pas capturer les raccourcis navigateur seuls hors canvas focus
  const canvas = document.getElementById("video-canvas");
  if (!canvas || document.activeElement !== canvas) return;
  e.preventDefault();
  void sendControl({
    type: "key_event",
    key: e.code,
    pressed,
    modifiers: {
      shift: e.shiftKey,
      ctrl: e.ctrlKey,
      alt: e.altKey,
      meta: e.metaKey,
    },
    timestamp_ms: nowMs(),
  });
}

function ensureKeyListeners(): void {
  if (keyListenerAttached) return;
  keyListenerAttached = true;
  window.addEventListener("keydown", (e) => onKeyEvent(e, true));
  window.addEventListener("keyup", (e) => onKeyEvent(e, false));
}

function isGuestSession(state: SessionState): boolean {
  return state.kind === "inSession" && state.role === "guest";
}

function isHostSession(state: SessionState): boolean {
  return (
    state.kind === "hosting" ||
    (state.kind === "inSession" && state.role === "host")
  );
}

function statusLabel(state: SessionState): string {
  switch (state.kind) {
    case "idle":
      return "Prêt";
    case "connecting":
      return "Connexion au relay…";
    case "hosting":
      return "En attente d’un pair…";
    case "inSession":
      return state.peerConnected
        ? "Pair connecté"
        : "En session — en attente du pair";
    case "error":
      return state.message;
  }
}

function decodeBase64Rgba(b64: string): Uint8ClampedArray {
  const bin = atob(b64);
  const buffer = new ArrayBuffer(bin.length);
  const out = new Uint8ClampedArray(buffer);
  for (let i = 0; i < bin.length; i += 1) {
    out[i] = bin.charCodeAt(i);
  }
  return out;
}

function paintFrame(frame: VideoFramePayload): void {
  const canvas = document.getElementById("video-canvas") as HTMLCanvasElement | null;
  if (!canvas) return;
  if (painting) {
    return;
  }
  painting = true;
  try {
    if (canvas.width !== frame.width || canvas.height !== frame.height) {
      canvas.width = frame.width;
      canvas.height = frame.height;
    }
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const pixels = decodeBase64Rgba(frame.rgbaBase64);
    const image = new ImageData(frame.width, frame.height);
    image.data.set(pixels);
    ctx.putImageData(image, 0, 0);
  } finally {
    painting = false;
  }
}

function render(): void {
  if (!root) return;

  if (model.screen === "join") {
    root.innerHTML = `
      <main class="shell">
        <h1 class="brand">Teleportal Remote</h1>
        <p class="lede">Entrez le code à 6 chiffres fourni par l’hôte.</p>
        <div class="stack">
          <label>
            Code de session
            <input id="join-code" inputmode="numeric" maxlength="6" placeholder="123456" value="${model.joinCode}" />
          </label>
          ${model.error ? `<p class="error">${model.error}</p>` : ""}
          <div class="actions">
            <button class="primary" id="btn-join" ${model.busy ? "disabled" : ""}>Rejoindre</button>
            <button class="ghost" id="btn-back" ${model.busy ? "disabled" : ""}>Retour</button>
          </div>
        </div>
      </main>
    `;
    document.getElementById("join-code")?.addEventListener("input", (e) => {
      const v = (e.target as HTMLInputElement).value.replace(/\D/g, "").slice(0, 6);
      model.joinCode = v;
      (e.target as HTMLInputElement).value = v;
    });
    document.getElementById("btn-back")?.addEventListener("click", () => {
      model.screen = "home";
      model.error = null;
      render();
    });
    document.getElementById("btn-join")?.addEventListener("click", () => void onJoin());
    return;
  }

  if (model.screen === "session") {
    const code =
      model.state.kind === "hosting"
        ? model.state.code
        : model.state.kind === "inSession"
          ? model.state.code
          : undefined;
    const guest = isGuestSession(model.state);
    const host = isHostSession(model.state);
    const videoBlock = guest
      ? `<div class="video-stage">
          <canvas id="video-canvas" class="video-canvas" width="16" height="9" aria-label="Flux vidéo distant"></canvas>
          ${remoteCursorMarkup()}
        </div>`
      : `<div class="video-stage video-stage-host">
          <div class="video-host">
            ${
              host && model.state.kind === "inSession" && model.state.peerConnected
                ? "Diffusion active — curseur distant sur votre bureau (overlay)"
                : "En attente de diffusion…"
            }
          </div>
        </div>`;

    root.innerHTML = `
      <main class="shell shell-session">
        <h1 class="brand">Teleportal Remote</h1>
        ${code ? `<p class="code">${code}</p>` : ""}
        <p class="status">${statusLabel(model.state)}</p>
        ${videoBlock}
        ${model.error ? `<p class="error">${model.error}</p>` : ""}
        <div class="actions">
          <button class="ghost" id="btn-leave" ${model.busy ? "disabled" : ""}>Quitter</button>
        </div>
      </main>
    `;
    document.getElementById("btn-leave")?.addEventListener("click", () => void onLeave());
    if (guest) {
      updateRemoteCursorMarker(lastRemoteCursor);
      ensureKeyListeners();
      const canvas = document.getElementById("video-canvas") as HTMLCanvasElement | null;
      if (canvas) {
        bindGuestControls(canvas);
        if (lastFrame) paintFrame(lastFrame);
      }
    }
    return;
  }

  root.innerHTML = `
    <main class="shell">
      <h1 class="brand">Teleportal Remote</h1>
      <p class="lede">Contrôle à distance collaboratif. Connectez-vous via un code à 6 chiffres.</p>
      <div class="stack">
        <label>
          URL du relay
          <input id="relay-url" value="${model.relayUrl}" />
        </label>
        ${model.error ? `<p class="error">${model.error}</p>` : ""}
        <div class="actions">
          <button class="primary" id="btn-host" ${model.busy ? "disabled" : ""}>Héberger</button>
          <button class="ghost" id="btn-goto-join" ${model.busy ? "disabled" : ""}>Rejoindre</button>
        </div>
        <p class="status">v${model.version || "…"}</p>
        ${model.updateStatus ? `<p class="status">${model.updateStatus}</p>` : ""}
      </div>
    </main>
  `;

  document.getElementById("relay-url")?.addEventListener("change", (e) => {
    model.relayUrl = (e.target as HTMLInputElement).value.trim();
  });
  document.getElementById("btn-host")?.addEventListener("click", () => void onHost());
  document.getElementById("btn-goto-join")?.addEventListener("click", () => {
    model.screen = "join";
    model.error = null;
    render();
  });
}

async function onHost(): Promise<void> {
  model.busy = true;
  model.error = null;
  const input = document.getElementById("relay-url") as HTMLInputElement | null;
  if (input) model.relayUrl = input.value.trim();
  render();
  try {
    const state = await invoke<SessionState>("host_start", {
      relayUrl: model.relayUrl,
    });
    model.state = state;
    model.screen = "session";
  } catch (e) {
    model.error = String(e);
    model.state = { kind: "idle" };
  } finally {
    model.busy = false;
    render();
  }
}

async function onJoin(): Promise<void> {
  model.busy = true;
  model.error = null;
  render();
  try {
    const state = await invoke<SessionState>("guest_join", {
      relayUrl: model.relayUrl,
      code: model.joinCode,
    });
    model.state = state;
    model.screen = "session";
  } catch (e) {
    model.error = String(e);
  } finally {
    model.busy = false;
    render();
  }
}

async function onLeave(): Promise<void> {
  model.busy = true;
  render();
  try {
    await invoke("leave_session");
    model.state = { kind: "idle" };
    model.screen = "home";
    model.error = null;
    lastFrame = null;
    clearRemoteCursor();
  } catch (e) {
    model.error = String(e);
    model.state = { kind: "idle" };
    model.screen = "home";
    lastFrame = null;
    clearRemoteCursor();
  } finally {
    model.busy = false;
    render();
  }
}

export async function startApp(): Promise<void> {
  try {
    model.version = await invoke<string>("app_version");
    const cfg = await invoke<ClientConfig>("get_config");
    model.relayUrl = cfg.relayUrl;
  } catch {
    // Browser preview without Tauri
    model.version = "preview";
  }

  await listen<SessionState>("session-state", (event) => {
    model.state = event.payload;
    if (event.payload.kind === "idle") {
      model.screen = "home";
      lastFrame = null;
      clearRemoteCursor();
    } else if (event.payload.kind === "error") {
      model.error = event.payload.message;
      model.screen = "home";
      model.state = { kind: "idle" };
      lastFrame = null;
      clearRemoteCursor();
    } else if (
      event.payload.kind === "inSession" &&
      !event.payload.peerConnected
    ) {
      clearRemoteCursor();
      model.screen = "session";
    } else {
      model.screen = "session";
    }
    render();
  });

  await listen<string>("session-error", (event) => {
    model.error = event.payload;
    render();
  });

  await listen<VideoFramePayload>("video-frame", (event) => {
    lastFrame = event.payload;
    if (model.screen === "session" && isGuestSession(model.state)) {
      paintFrame(event.payload);
    }
  });

  await listen<RemoteCursorPayload>("remote-cursor", (event) => {
    lastRemoteCursor = event.payload;
    if (model.screen === "session" && isGuestSession(model.state)) {
      updateRemoteCursorMarker(event.payload);
    }
  });

  render();
  void checkAndApplyUpdate();
}

/** Check update canal stable ; download + install auto si disponible. */
async function checkAndApplyUpdate(): Promise<void> {
  if (model.version === "preview") return;
  try {
    const update = await check();
    if (!update) return;
    model.updateStatus = `Mise à jour ${update.version}…`;
    render();
    await update.downloadAndInstall();
    model.updateStatus = "Redémarrage…";
    render();
    await relaunch();
  } catch {
    // Non bloquant (hors ligne, pas de release, preview navigateur, etc.)
    model.updateStatus = null;
    if (model.screen === "home") render();
  }
}
