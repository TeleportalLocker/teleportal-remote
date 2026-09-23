import { listen } from "@tauri-apps/api/event";

type RemoteCursorPayload = {
  cursorId: string;
  x: number;
  y: number;
  timestampMs: number;
};

function updateMarker(pose: RemoteCursorPayload | null): void {
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

/** Fenêtre Host transparente plein écran (click-through). */
export async function startOverlayApp(): Promise<void> {
  document.documentElement.classList.add("overlay-mode");
  document.body.classList.add("overlay-mode");

  const root = document.querySelector<HTMLDivElement>("#app");
  if (!root) {
    throw new Error("#app missing");
  }

  root.innerHTML = `<div class="overlay-stage"><div id="remote-cursor" class="remote-cursor" hidden aria-hidden="true"></div></div>`;

  await listen<RemoteCursorPayload>("remote-cursor", (event) => {
    updateMarker(event.payload);
  });
}
