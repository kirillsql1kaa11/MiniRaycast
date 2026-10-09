import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { LauncherItem, AppearanceSettings } from "./types";
import { ICONS, getItemIcon } from "./icons";

let items: LauncherItem[] = [];
let selectedIndex = 0;
let debounceTimer: number | null = null;
let toastTimer: number | null = null;
let currentSearchQuery = "";
let isPreviewCollapsed = false;

const searchInput = document.getElementById("search-input") as HTMLInputElement;
const resultsList = document.getElementById("results-list") as HTMLDivElement;
const emptyState = document.getElementById("empty-state") as HTMLDivElement;
const metaCount = document.getElementById("meta-count") as HTMLSpanElement;
const actionLabel = document.getElementById("action-label") as HTMLSpanElement;
const hotkeyPill = document.getElementById("hotkey-pill") as HTMLSpanElement;
const themePill = document.getElementById("theme-pill") as HTMLSpanElement;
const toast = document.getElementById("toast") as HTMLDivElement;
const toastText = document.getElementById("toast-text") as HTMLSpanElement;
const toastIcon = document.getElementById("toast-icon") as HTMLSpanElement;
const searchIconSlot = document.getElementById("search-icon-slot") as HTMLDivElement;
const emptyIconSlot = document.getElementById("empty-icon-slot") as HTMLDivElement;
const previewPanel = document.getElementById("preview-panel") as HTMLElement;
const previewIconBox = document.getElementById("preview-icon-box") as HTMLDivElement;
const previewTitle = document.getElementById("preview-title") as HTMLDivElement;
const previewSubtitle = document.getElementById("preview-subtitle") as HTMLDivElement;
const previewBody = document.getElementById("preview-body") as HTMLDivElement;
const previewActionBtn = document.getElementById("preview-action-btn") as HTMLButtonElement;
const previewActionBtnText = document.getElementById("preview-action-btn-text") as HTMLSpanElement;
const previewTogglePill = document.getElementById("preview-toggle-pill") as HTMLDivElement;

searchIconSlot.innerHTML = ICONS.search;
emptyIconSlot.innerHTML = ICONS.empty;
toastIcon.innerHTML = ICONS.check;

async function initAppearance() {
  try {
    const settings = await invoke<AppearanceSettings>("get_appearance_settings");
    applyAppearance(settings.theme, settings.opacity, settings.blur);
  } catch {
    applyAppearance("oled", "94", "32");
  }
}

function applyAppearance(theme: string, opacity: string, blur: string) {
  document.body.dataset.theme = theme;
  if (themePill) {
    themePill.textContent = theme.toUpperCase();
  }

  const opNum = parseInt(opacity, 10);
  const opVal = isNaN(opNum) ? 0.94 : opNum / 100;
  document.documentElement.style.setProperty("--bg-opacity", opVal.toString());

  const blurNum = parseInt(blur, 10);
  const blurVal = isNaN(blurNum) ? 32 : blurNum;
  document.documentElement.style.setProperty("--bg-blur", `${blurVal}px`);
}

listen<{ key: string; value: string }>("appearance-changed", (event) => {
  const { key, value } = event.payload;
  if (key === "theme") {
    document.body.dataset.theme = value;
    if (themePill) {
      themePill.textContent = value.toUpperCase();
    }
  } else if (key === "opacity") {
    const opNum = parseInt(value, 10);
    const opVal = isNaN(opNum) ? 0.94 : opNum / 100;
    document.documentElement.style.setProperty("--bg-opacity", opVal.toString());
  } else if (key === "blur") {
    const blurNum = parseInt(value, 10);
    const blurVal = isNaN(blurNum) ? 32 : blurNum;
    document.documentElement.style.setProperty("--bg-blur", `${blurVal}px`);
  }
});

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
  }, 2200);
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

function togglePreviewPanel() {
  isPreviewCollapsed = !isPreviewCollapsed;
  if (isPreviewCollapsed) {
    previewPanel.classList.add("collapsed");
  } else {
    previewPanel.classList.remove("collapsed");
    renderPreview();
  }
}

previewTogglePill.addEventListener("click", () => {
  togglePreviewPanel();
});

function updateActionLabel() {
  const current = items[selectedIndex];
  if (!current) {
    actionLabel.textContent = "Открыть";
    if (previewActionBtnText) previewActionBtnText.textContent = "Открыть";
    return;
  }
  if (current.action === "kill_process") {
    actionLabel.textContent = "Завершить";
    if (previewActionBtnText) previewActionBtnText.textContent = "Завершить процесс";
    previewActionBtn.className = "preview-action-btn danger";
  } else if (current.action === "network_control") {
    actionLabel.textContent = "Выполнить";
    if (previewActionBtnText) previewActionBtnText.textContent = "Выполнить команду";
    previewActionBtn.className = "preview-action-btn";
  } else if (current.action === "copy") {
    actionLabel.textContent = "Скопировать";
    if (previewActionBtnText) previewActionBtnText.textContent = "Скопировать";
    previewActionBtn.className = "preview-action-btn";
  } else if (current.action === "set_theme") {
    actionLabel.textContent = "Применить";
    if (previewActionBtnText) previewActionBtnText.textContent = "Применить тему";
    previewActionBtn.className = "preview-action-btn";
  } else if (current.action === "timer") {
    actionLabel.textContent = "Запустить";
    if (previewActionBtnText) previewActionBtnText.textContent = "Запустить таймер";
    previewActionBtn.className = "preview-action-btn";
  } else if (current.action === "set_hotkey") {
    actionLabel.textContent = "Назначить";
    if (previewActionBtnText) previewActionBtnText.textContent = "Назначить хоткей";
    previewActionBtn.className = "preview-action-btn";
  } else if (current.action === "fill_search") {
    actionLabel.textContent = "Выбрать";
    if (previewActionBtnText) previewActionBtnText.textContent = "Открыть раздел";
    previewActionBtn.className = "preview-action-btn";
  } else if (current.item_type === "system") {
    actionLabel.textContent = "Выполнить";
    if (previewActionBtnText) previewActionBtnText.textContent = "Выполнить";
    previewActionBtn.className = "preview-action-btn";
  } else {
    actionLabel.textContent = "Открыть";
    if (previewActionBtnText) previewActionBtnText.textContent = "Запустить";
    previewActionBtn.className = "preview-action-btn";
  }
}

function renderPreview() {
  if (isPreviewCollapsed) return;

  const item = items[selectedIndex];
  if (!item) {
    previewIconBox.innerHTML = ICONS.search;
    previewTitle.textContent = "Нет элемента";
    previewSubtitle.textContent = "Выберите элемент из списка";
    previewBody.innerHTML = "";
    return;
  }

  const iconSvg = getItemIcon(item.item_type, item.action, item.payload);
  previewIconBox.innerHTML = iconSvg;
  previewTitle.textContent = item.title;
  previewSubtitle.textContent = item.subtitle;

  let bodyHtml = "";

  if (item.action === "kill_process" || item.item_type === "process") {
    const kws = item.keywords || [];
    const ramStr = kws[1] ? `${kws[1]} МБ` : "Неизвестно";
    const cpuStr = kws[2] ? `${kws[2]}%` : "0.0%";
    const procPath = kws[3] || "Системный процесс";
    const ramNum = parseFloat(kws[1] || "0");
    const ramPercent = Math.min(Math.round((ramNum / 2048) * 100), 100);

    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Потребление ресурсов</div>
        <div class="preview-row">
          <span class="preview-label">Память (RAM)</span>
          <span class="preview-value highlight">${escapeHtml(ramStr)}</span>
        </div>
        <div class="preview-meter-container">
          <div class="preview-meter-bar">
            <div class="preview-meter-fill ${ramNum > 1000 ? "danger" : ramNum > 500 ? "warn" : ""}" style="width: ${ramPercent}%;"></div>
          </div>
        </div>
        <div class="preview-row" style="margin-top: 6px;">
          <span class="preview-label">Нагрузка CPU</span>
          <span class="preview-value">${escapeHtml(cpuStr)}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Идентификатор (PID)</span>
          <span class="preview-value mono">${escapeHtml(item.payload)}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Статус процесса</span>
          <span class="preview-value" style="color: var(--accent-green);">Активен</span>
        </div>
      </div>
      <div class="preview-section">
        <div class="preview-section-title">Расположение файла</div>
        <div class="preview-row" style="flex-direction: column; align-items: flex-start; gap: 4px;">
          <span class="preview-value mono" style="max-width: 100%; white-space: normal; word-break: break-all; text-align: left;">
            ${escapeHtml(procPath)}
          </span>
        </div>
      </div>
    `;
  } else if (item.item_type === "network") {
    let cmd = "PowerShell";
    let desc = item.subtitle;

    if (item.payload === "flushdns") {
      cmd = "ipconfig /flushdns";
      desc = "Очистка кэша DNS-клиента Windows для исправления сетевых адресов";
    } else if (item.payload.startsWith("ping")) {
      const host = item.payload.replace("ping:", "");
      cmd = `Test-Connection -ComputerName ${host} -Count 1`;
      desc = `Проверка задержки и доступности интернет-узла ${host}`;
    } else if (item.payload.includes("get_ip")) {
      cmd = item.payload.includes("external") ? "api.ipify.org" : "Get-NetIPAddress";
      desc = "Определение активного IP-адреса и копирование в буфер";
    } else if (item.payload.includes("bluetooth")) {
      cmd = "PowerShell: bthserv";
      desc = "Управление службой и состоянием адаптера Bluetooth";
    }

    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Параметры сетевой команды</div>
        <div class="preview-row">
          <span class="preview-label">Команда</span>
          <span class="preview-value mono">${escapeHtml(cmd)}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Тип действия</span>
          <span class="preview-value">Системная утилита</span>
        </div>
      </div>
      <div class="preview-section">
        <div class="preview-section-title">Назначение</div>
        <div class="preview-row" style="flex-direction: column; align-items: flex-start; gap: 4px;">
          <span class="preview-label" style="line-height: 1.4; color: var(--text-main);">
            ${escapeHtml(desc)}
          </span>
        </div>
      </div>
    `;
  } else if (item.item_type === "theme") {
    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Цветовая палитра</div>
        <div class="preview-row">
          <span class="preview-label">Пресет</span>
          <span class="preview-value highlight">${escapeHtml(item.payload.toUpperCase())}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Описание</span>
          <span class="preview-value">${escapeHtml(item.subtitle)}</span>
        </div>
      </div>
    `;
  } else if (item.item_type === "timer") {
    const secs = parseInt(item.payload, 10) || 60;
    const mins = Math.round(secs / 60);
    const finishDate = new Date(Date.now() + secs * 1000);
    const finishTime = finishDate.toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit", second: "2-digit" });

    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Параметры таймера</div>
        <div class="preview-row">
          <span class="preview-label">Длительность</span>
          <span class="preview-value highlight">${secs} сек. (${mins} мин.)</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Время окончания</span>
          <span class="preview-value mono">${finishTime}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Оповещение</span>
          <span class="preview-value">Всплывающее окно</span>
        </div>
      </div>
    `;
  } else if (item.item_type === "app") {
    const isLnk = item.payload.toLowerCase().endsWith(".lnk");
    const isExe = item.payload.toLowerCase().endsWith(".exe");
    const isUrl = item.payload.toLowerCase().endsWith(".url");
    const kind = isLnk ? "Ярлык программы (.lnk)" : isExe ? "Приложение (.exe)" : isUrl ? "Интернет-ярлык (.url)" : "Файл";

    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Сведения о программе</div>
        <div class="preview-row">
          <span class="preview-label">Тип</span>
          <span class="preview-value">${kind}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Действие</span>
          <span class="preview-value">Запуск процесса</span>
        </div>
      </div>
      <div class="preview-section">
        <div class="preview-section-title">Расположение</div>
        <div class="preview-row" style="flex-direction: column; align-items: flex-start; gap: 4px;">
          <span class="preview-value mono" style="max-width: 100%; white-space: normal; word-break: break-all; text-align: left;">
            ${escapeHtml(item.payload)}
          </span>
        </div>
      </div>
    `;
  } else if (item.action === "copy") {
    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Значение для копирования</div>
        <div class="preview-row" style="flex-direction: column; align-items: flex-start; gap: 6px;">
          <span class="preview-value mono highlight" style="max-width: 100%; font-size: 14px; white-space: normal; word-break: break-all; text-align: left;">
            ${escapeHtml(item.payload)}
          </span>
        </div>
        <div class="preview-row" style="margin-top: 6px;">
          <span class="preview-label">Запрос</span>
          <span class="preview-value">${escapeHtml(item.subtitle)}</span>
        </div>
      </div>
    `;
  } else {
    bodyHtml = `
      <div class="preview-section">
        <div class="preview-section-title">Информация</div>
        <div class="preview-row">
          <span class="preview-label">Тип</span>
          <span class="preview-value">${escapeHtml(item.badge || item.item_type)}</span>
        </div>
        <div class="preview-row">
          <span class="preview-label">Действие</span>
          <span class="preview-value">${escapeHtml(item.action)}</span>
        </div>
      </div>
      <div class="preview-section">
        <div class="preview-section-title">Параметры</div>
        <div class="preview-row" style="flex-direction: column; align-items: flex-start; gap: 4px;">
          <span class="preview-value mono" style="max-width: 100%; white-space: normal; word-break: break-all; text-align: left;">
            ${escapeHtml(item.payload)}
          </span>
        </div>
      </div>
    `;
  }

  previewBody.innerHTML = bodyHtml;
}

previewActionBtn.addEventListener("click", () => {
  executeSelectedItem();
});

function renderItems() {
  resultsList.innerHTML = "";

  if (items.length === 0) {
    resultsList.style.display = "none";
    emptyState.style.display = "flex";
    metaCount.textContent = "0 элементов";
    updateActionLabel();
    renderPreview();
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
  renderPreview();
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
  renderPreview();
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
    return;
  }

  if (item.action === "fill_search") {
    searchInput.value = item.payload;
    searchInput.focus();
    fetchSearch(item.payload);
    return;
  }

  if (item.action === "timer") {
    const secs = parseInt(item.payload, 10);
    showToast(`Таймер запущен на ${secs} сек.`);
    return;
  }

  if (item.action === "set_hotkey") {
    showToast(`Хоткей: ${item.payload}`);
    try {
      await invoke("execute_item", { item, query: searchInput.value });
      refreshHotkey();
    } catch {
      showToast("Ошибка смены хоткея");
    }
    return;
  }

  if (item.action === "set_theme") {
    applyAppearance(item.payload, "94", "32");
    showToast(`Применена тема: ${item.title}`);
    try {
      await invoke("execute_item", { item, query: searchInput.value });
    } catch {
      showToast("Ошибка сохранения темы");
    }
    return;
  }

  if (item.action === "kill_process") {
    showToast(`Завершение процесса ${item.title}...`);
    try {
      await invoke("kill_process_by_pid", { pid: item.payload });
      showToast(`Процесс завершен: ${item.title}`);
      fetchSearch(searchInput.value);
    } catch (e) {
      showToast(`Ошибка: ${e}`);
    }
    return;
  }

  if (item.action === "network_control") {
    const parts = item.payload.split(":");
    const action = parts[0];
    const target = parts.length > 1 ? parts[1] : "";

    showToast("Выполнение команды...");
    try {
      const resStr = await invoke<string>("run_network_command", { action, target: target || null });
      try {
        const json = JSON.parse(resStr);
        if (action === "flushdns") {
          showToast("DNS-кэш успешно очищен");
        } else if (action === "ping") {
          const time = json.time_ms;
          const status = json.status;
          showToast(`Пинг ${json.host}: ${time >= 0 ? time + " мс (" + status + ")" : status}`);
          if (time >= 0) {
            await navigator.clipboard.writeText(`${json.host}: ${time} ms`);
          }
        } else if (action.startsWith("get_ip")) {
          const isExt = item.payload.includes("external");
          const ipVal = isExt ? json.external_ip : json.local_ip;
          await navigator.clipboard.writeText(ipVal);
          showToast(`IP скопирован: ${ipVal}`);
        } else if (action.startsWith("bluetooth")) {
          showToast(json.message || "Bluetooth переключен");
        } else {
          showToast(json.message || "Выполнено успешно");
        }
      } catch {
        showToast("Команда выполнена");
      }
    } catch {
      showToast("Ошибка выполнения сетевой команды");
    }
    return;
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
  }, 30);
});

window.addEventListener("keydown", (e: KeyboardEvent) => {
  if (e.key === "p" && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    togglePreviewPanel();
    return;
  }

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
  initAppearance();
});

initAppearance();
refreshHotkey();
fetchSearch("");
searchInput.focus();
