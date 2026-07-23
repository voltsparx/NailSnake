const REPO = "voltsparx/NailSnake";
const API_BASE = `https://api.github.com/repos/${REPO}`;
const STORAGE_KEY = "nailsnake-theme";

const formatNumber = new Intl.NumberFormat();
const releaseTypes = [
  ".deb",
  ".rpm",
  ".pkg.tar.zst",
  ".zst",
  ".msi",
  ".pkg",
  ".tar.gz",
  ".zip",
  ".exe"
];

initTheme();
loadRepositoryData();
loadReleaseAssets();
startOxideCanvas();

function initTheme() {
  const saved = localStorage.getItem(STORAGE_KEY) || "system";
  applyTheme(saved);

  document.querySelectorAll("[data-theme]").forEach((button) => {
    button.addEventListener("click", () => {
      const theme = button.dataset.theme;
      localStorage.setItem(STORAGE_KEY, theme);
      applyTheme(theme);
    });
  });
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme === "system" ? "" : theme;
  document.querySelectorAll("[data-theme]").forEach((button) => {
    button.classList.toggle("active", button.dataset.theme === theme);
  });
}

async function loadRepositoryData() {
  const stars = document.getElementById("repo-stars");
  if (!stars) return;

  try {
    const [repo, release] = await Promise.all([
      fetchJson(API_BASE),
      fetchJson(`${API_BASE}/releases/latest`).catch(() => null)
    ]);

    stars.textContent = formatNumber.format(repo.stargazers_count || 0);
    setText("repo-version", release?.tag_name || "v1.0");
    setText("repo-updated", formatDate(repo.pushed_at));
    setText("repo-license", repo.license?.spdx_id || "MIT");
  } catch (error) {
    stars.textContent = "Live";
    setText("repo-version", "v1.0");
    setText("repo-updated", "GitHub");
  }
}

async function loadReleaseAssets() {
  const list = document.getElementById("release-assets");
  if (!list) return;

  try {
    const release = await fetchJson(`${API_BASE}/releases/latest`);
    setText("download-version", release.tag_name || "Latest");

    const assets = (release.assets || []).filter((asset) =>
      releaseTypes.some((type) => asset.name.toLowerCase().endsWith(type))
    );

    if (!assets.length) {
      list.innerHTML = `
        <p class="muted">No package assets are attached to the latest release yet.</p>
        <a class="button primary" href="${release.html_url}">Open latest release</a>
      `;
      return;
    }

    list.innerHTML = assets.map(renderAsset).join("");
  } catch (error) {
    setText("download-version", "Unavailable");
    list.innerHTML = `
      <p class="muted">Could not load release assets from GitHub right now.</p>
      <a class="button primary" href="https://github.com/${REPO}/releases">Open releases</a>
    `;
  }
}

function renderAsset(asset) {
  return `
    <a class="asset" href="${asset.browser_download_url}">
      <span>
        <strong>${escapeHtml(asset.name)}</strong>
        <small>${formatBytes(asset.size)} · ${asset.download_count || 0} downloads</small>
      </span>
      <span class="button">Download</span>
    </a>
  `;
}

async function fetchJson(url) {
  const response = await fetch(url, {
    headers: { Accept: "application/vnd.github+json" }
  });
  if (!response.ok) throw new Error(`GitHub request failed: ${response.status}`);
  return response.json();
}

function setText(id, value) {
  const element = document.getElementById(id);
  if (element) element.textContent = value;
}

function formatDate(value) {
  if (!value) return "Unknown";
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric"
  }).format(new Date(value));
}

function formatBytes(bytes) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function escapeHtml(value) {
  return value.replace(/[&<>"']/g, (char) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    "\"": "&quot;",
    "'": "&#039;"
  })[char]);
}

function startOxideCanvas() {
  const canvas = document.getElementById("oxide-canvas");
  if (!canvas) return;

  const context = canvas.getContext("2d");
  const glyphs = ["0", "1", "R", "s", ">", "{", "}", "::", "fn"];
  let columns = [];

  function resize() {
    const rect = canvas.getBoundingClientRect();
    const scale = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(rect.width * scale));
    canvas.height = Math.max(1, Math.floor(rect.height * scale));
    context.setTransform(scale, 0, 0, scale, 0, 0);

    const count = Math.max(12, Math.floor(rect.width / 34));
    columns = Array.from({ length: count }, (_, index) => ({
      x: (index / count) * rect.width,
      y: Math.random() * rect.height,
      speed: 0.25 + Math.random() * 0.8,
      phase: Math.random() * Math.PI
    }));
  }

  function frame(time) {
    const rect = canvas.getBoundingClientRect();
    context.clearRect(0, 0, rect.width, rect.height);
    context.font = "700 15px Cascadia Mono, Consolas, monospace";

    columns.forEach((column, index) => {
      column.y += column.speed;
      if (column.y > rect.height + 80) column.y = -40;

      for (let row = 0; row < 9; row += 1) {
        const alpha = Math.max(0, 0.72 - row * 0.08);
        const y = column.y - row * 22;
        const wobble = Math.sin(time / 900 + column.phase + row) * 8;
        const glyph = glyphs[(index + row + Math.floor(time / 700)) % glyphs.length];
        context.fillStyle = row === 0
          ? `rgba(255, 177, 95, ${alpha})`
          : `rgba(201, 87, 42, ${alpha})`;
        context.fillText(glyph, column.x + wobble, y);
      }
    });

    requestAnimationFrame(frame);
  }

  resize();
  window.addEventListener("resize", resize);
  requestAnimationFrame(frame);
}
