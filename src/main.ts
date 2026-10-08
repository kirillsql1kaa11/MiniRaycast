import { invoke } from "@tauri-apps/api/core";
import { LauncherItem } from "./types";
import { ICONS, getItemIcon } from "./icons";

let items: LauncherItem[] = [];
let selectedIndex = 0;
let debounceTimer: number | null = null;
let toastTimer: number | null = null;
let currentSearchQuery = "";

const searchInput = document.getElementById("search-input") as HTMLInputElement;
const resultsList = document.getElementById("results-list") as HTMLDivElement;
const emptyState = document.getElementById("empty-state") as HTMLDivElement;
const metaCount = document.getElementById("meta-count") as HTMLSpanElement;
const actionLabel = document.getElementById("action-label") as HTMLSpanElement;
const hotkeyPill = document.getElementById("hotkey-pill") as HTMLSpanElement;
const toast = document.getElementById("toast") as HTMLDivElement;
const toastText = document.getElementById("toast-text") as HTMLSpanElement;
const toastIcon = document.getElementById("toast-icon") as HTMLSpanElement;
const searchIconSlot = document.getElementById("search-icon-slot") as HTMLDivElement;
const emptyIconSlot = document.getElementById("empty-icon-slot") as HTMLDivElement;

searchIconSlot.innerHTML = ICONS.search;
emptyIconSlot.innerHTML = ICONS.empty;
toastIcon.innerHTML = ICONS.check;

async function refreshHotkey() {
  try {
    const hk = await invoke<string>("get_current_hotkey");
    if (hk && hotkeyPill) {
      hotkeyPill.textContent = hk;
    }
  } catch {
    if (hotkeyPill) {
      hotkeyPill.textContent = "Alt+Space";
    }
  }
}

function showToast(message: string) {
  if (toastTimer) clearTimeout(toastTimer);
  toastText.textContent = message;
  toast.style.display = "flex";
  toastTimer = window.setTimeout(() => {
    toast.style.display = "none";
  }, 1800);
}

function formatItemCount(count: number): string {
  if (count % 10 === 1 && count % 100 !== 11) {
    return `${count} элемент`;
  }
  if ([2, 3, 4].includes(count % 10) && ![12, 13, 14].includes(count % 100)) {
    return `${count} элемента`;
  }
  return `${count} элементов`;
}

function updateActionLabel() {
  const current = items[selectedIndex];
  if (!current) {
    actionLabel.textContent = "Открыть";
    return;
  }
  if (current.action === "copy") {
    actionLabel.textContent = "Скопировать";
  } else if (current.action === "timer") {
    actionLabel.textContent = "Запустить";
  } else if (current.action === "set_hotkey") {
    actionLabel.textContent = "Назначить";
  } else if (current.item_type === "system") {
    actionLabel.textContent = "Выполнить";
  } else {
    actionLabel.textContent = "Открыть";
  }
}

function renderItems() {
  resultsList.innerHTML = "";

  if (items.length === 0) {
    resultsList.style.display = "none";
    emptyState.style.display = "flex";
    metaCount.textContent = "0 элементов";
    updateActionLabel();
    return;
  }

  emptyState.style.display = "none";
  resultsList.style.display = "flex";
  metaCount.textContent = formatItemCount(items.length);

  items.forEach((item, index) => {
    const itemEl = document.createElement("div");
    itemEl.className = `result-item ${index === selectedIndex ? "selected" : ""}`;

    const iconSvg = getItemIcon(item.item_type, item.action, item.payload);

    itemEl.innerHTML = `
      <div class="item-icon-box">${iconSvg}</div>
      <div class="item-content">
        <div class="item-title">${escapeHtml(item.title)}</div>
        <div class="item-subtitle">${escapeHtml(item.subtitle)}</div>
      </div>
      <div class="item-badge-box">
        ${item.badge ? `<span class="item-badge">${escapeHtml(item.badge)}</span>` : ""}
      </div>
    `;

    itemEl.addEventListener("click", () => {
      selectedIndex = index;
      renderSelection();
      executeSelectedItem();
    });

    itemEl.addEventListener("mouseenter", () => {
      selectedIndex = index;
      renderSelection();
    });

    resultsList.appendChild(itemEl);
  });

  updateActionLabel();
  scrollSelectedIntoView();
}

function renderSelection() {
  const children = resultsList.children;
  for (let i = 0; i < children.length; i++) {
    if (i === selectedIndex) {
      children[i].classList.add("selected");
    } else {
      children[i].classList.remove("selected");
    }
  }
  updateActionLabel();
  scrollSelectedIntoView();
}

function scrollSelectedIntoView() {
  const selectedEl = resultsList.children[selectedIndex] as HTMLElement | undefined;
  if (selectedEl) {
    selectedEl.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

async function fetchSearch(query: string) {
  currentSearchQuery = query;
  try {
    const res = await invoke<LauncherItem[]>("search", { query });
    if (currentSearchQuery !== query) {
      return;
    }
    items = res;
    selectedIndex = 0;
    renderItems();
  } catch {
    if (currentSearchQuery !== query) {
      return;
    }
    items = [];
    selectedIndex = 0;
    renderItems();
  }
}

async function executeSelectedItem() {
  const item = items[selectedIndex];
  if (!item) return;

  if (item.action === "copy") {
    try {
      await navigator.clipboard.writeText(item.payload);
      showToast("Скопировано в буфер");
    } catch {
      showToast("Не удалось скопировать");
    }
  } else if (item.action === "timer") {
    const secs = parseInt(item.payload, 10);
    showToast(`Таймер запущен на ${secs} сек.`);
  } else if (item.action === "set_hotkey") {
    showToast(`Хоткей: ${item.payload}`);
    refreshHotkey();
  }

  try {
    await invoke("execute_item", {
      item,
      query: searchInput.value,
    });
  } catch {
    showToast("Ошибка запуска");
  }
}

searchInput.addEventListener("input", () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = window.setTimeout(() => {
    fetchSearch(searchInput.value);
  }, 25);
});

window.addEventListener("keydown", (e: KeyboardEvent) => {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (items.length > 0) {
      selectedIndex = (selectedIndex + 1) % items.length;
      renderSelection();
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (items.length > 0) {
      selectedIndex = (selectedIndex - 1 + items.length) % items.length;
      renderSelection();
    }
  } else if (e.key === "Enter") {
    e.preventDefault();
    executeSelectedItem();
  } else if (e.key === "Escape") {
    e.preventDefault();
    if (searchInput.value.length > 0) {
      searchInput.value = "";
      fetchSearch("");
    } else {
      try {
        invoke("hide_window");
      } catch {
        showToast("Окно скрыто");
      }
    }
  }
});

window.addEventListener("focus", () => {
  searchInput.focus();
  refreshHotkey();
});

refreshHotkey();
fetchSearch("");
searchInput.focus();
