// ─── Tauri API helpers ───
const invoke = window.__TAURI__.core.invoke;
const listen = window.__TAURI__.event.listen;

// ─── State ───
let selectedHost = null;
let selectedFiles = [];
let serverRunning = false;

// ─── DOM refs ───
const el = (id) => document.getElementById(id);

// ─── Toast ───
let toastTimer;
function showToast(msg, type) {
  const e = el("toast");
  e.textContent = msg;
  e.className = "toast " + (type || "");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => e.classList.add("hidden"), 3500);
}

// ─── Init ───
async function init() {
  document.querySelectorAll(".tab").forEach((btn) => {
    btn.addEventListener("click", () => switchTab(btn.dataset.tab));
  });

  el("btn-start-server").addEventListener("click", startServer);
  el("btn-stop-server").addEventListener("click", stopServer);
  el("btn-discover").addEventListener("click", discoverHosts);
  el("btn-pick-files").addEventListener("click", pickFiles);
  el("btn-clear-files").addEventListener("click", clearFiles);
  el("btn-send").addEventListener("click", sendFiles);
  el("btn-clear-history").addEventListener("click", clearHistory);

  await listen("server-status", (e) => {
    serverRunning = e.payload.running;
    updateServerUI(e.payload);
  });

  await listen("transfer-request", (e) => {
    addHistory(e.payload);
  });

  await listen("transfer-progress", (e) => {
    updateProgress(e.payload);
  });

  try {
    const name = await invoke("get_hostname");
    el("hostname").textContent = name;
  } catch (e) {
    el("hostname").textContent = "unknown";
  }

  try {
    await invoke("start_broadcast");
  } catch (e) {}
}

// ─── Tab switching ───
function switchTab(tab) {
  document
    .querySelectorAll(".tab")
    .forEach((b) => b.classList.remove("active"));
  document.querySelector('[data-tab="' + tab + '"]').classList.add("active");
  document
    .querySelectorAll(".tab-content")
    .forEach((s) => s.classList.remove("active"));
  el("tab-" + tab).classList.add("active");
}

// ─── Server ───
async function startServer() {
  el("btn-start-server").disabled = true;
  try {
    const msg = await invoke("start_server");
    showToast(msg, "success");
  } catch (e) {
    showToast("Failed: " + e, "error");
    el("btn-start-server").disabled = false;
  }
}

async function stopServer() {
  el("btn-stop-server").disabled = true;
  try {
    const msg = await invoke("stop_server");
    showToast(msg, "success");
  } catch (e) {
    showToast("Failed: " + e, "error");
    el("btn-stop-server").disabled = false;
  }
}

function updateServerUI(info) {
  const dot = el("server-dot");
  const addr = el("server-addr");
  const startBtn = el("btn-start-server");
  const stopBtn = el("btn-stop-server");

  if (info.running) {
    dot.classList.add("on");
    addr.textContent = info.address;
    startBtn.disabled = true;
    stopBtn.disabled = false;
  } else {
    dot.classList.remove("on");
    addr.textContent = "not running";
    startBtn.disabled = false;
    stopBtn.disabled = true;
  }
}

// ─── Discovery ───
async function discoverHosts() {
  const list = el("hosts-list");
  list.innerHTML = '<p class="muted">Scanning network...</p>';
  try {
    const hosts = await invoke("discover_hosts");
    if (hosts.length === 0) {
      list.innerHTML = '<p class="muted">No hosts found.</p>';
      return;
    }
    list.innerHTML = "";
    hosts.forEach((h) => {
      const div = document.createElement("div");
      div.className = "host-item";
      if (selectedHost && selectedHost.ip === h.ip)
        div.classList.add("selected");
      div.innerHTML =
        '<span class="name">' +
        h.name +
        '</span><span class="ip">' +
        h.ip +
        "</span>";
      div.addEventListener("click", () => selectHost(h, div));
      list.appendChild(div);
    });
  } catch (e) {
    list.innerHTML = '<p class="muted">Discovery failed: ' + e + "</p>";
  }
}

function selectHost(host, elDiv) {
  document
    .querySelectorAll(".host-item")
    .forEach((i) => i.classList.remove("selected"));
  elDiv.classList.add("selected");
  selectedHost = host;
  updateSendButton();
}

// ─── Files ───
async function pickFiles() {
  try {
    const files = await invoke("pick_files");
    if (files.length > 0) {
      selectedFiles = [...new Set([...selectedFiles, ...files])];
      renderFiles();
    }
  } catch (e) {
    showToast("File pick failed: " + e, "error");
  }
}

function clearFiles() {
  selectedFiles = [];
  renderFiles();
}

function renderFiles() {
  const list = el("files-list");
  if (selectedFiles.length === 0) {
    list.innerHTML = '<p class="muted">No files selected</p>';
  } else {
    list.innerHTML = "";
    selectedFiles.forEach((path, i) => {
      const div = document.createElement("div");
      div.className = "file-item";
      const name = path.replace(/^.*[\\\\/]/, "");
      div.innerHTML =
        '<span class="path">' +
        name +
        '</span><button class="btn btn-small btn-red">X</button>';
      div.querySelector("button").addEventListener("click", (e) => {
        e.stopPropagation();
        selectedFiles.splice(i, 1);
        renderFiles();
        updateSendButton();
      });
      list.appendChild(div);
    });
  }
  updateSendButton();
}

function updateSendButton() {
  el("btn-send").disabled = !(selectedHost && selectedFiles.length > 0);
}

// ─── Send ───
async function sendFiles() {
  if (!selectedHost || selectedFiles.length === 0) return;

  const hostAddr = selectedHost.ip + ":8080";
  el("btn-send").disabled = true;
  el("send-progress").classList.remove("hidden");
  el("progress-fill").style.width = "0%";
  el("progress-text").textContent = "Connecting...";

  try {
    const msg = await invoke("send_files", {
      host: hostAddr,
      filePaths: selectedFiles,
    });
    showToast(msg, "success");
    el("progress-text").textContent = "Done!";
  } catch (e) {
    showToast("Send failed: " + e, "error");
    el("progress-text").textContent = "Failed: " + e;
    el("btn-send").disabled = false;
  }
}

function updateProgress(info) {
  if (info.step === "prepared") {
    el("progress-text").textContent = "Transfer accepted, uploading...";
    el("progress-fill").style.width = "5%";
  } else if (info.step === "uploading") {
    const pct = Math.round((info.current / info.total) * 90) + 5;
    el("progress-fill").style.width = pct + "%";
    el("progress-text").textContent =
      "Sending " + info.file + " (" + info.current + "/" + info.total + ")";
  } else if (info.step === "done") {
    el("progress-fill").style.width = "100%";
    el("progress-text").textContent = "All files sent!";
    el("btn-send").disabled = false;
  }
}

// ─── History ───
function addHistory(info) {
  const list = el("history-list");
  if (list.querySelector(".muted")) list.innerHTML = "";
  const div = document.createElement("div");
  div.className = "history-item";
  const now = new Date().toLocaleTimeString();
  const fileCount = info.files ? info.files.length : 0;
  div.innerHTML =
    '<span class="icon">inbox</span><span class="desc"><strong>' +
    info.sender_name +
    "</strong> wants to send " +
    fileCount +
    ' file(s)</span><span class="time">' +
    now +
    "</span>";
  list.prepend(div);
}

function clearHistory() {
  el("history-list").innerHTML = '<p class="muted">No transfers yet</p>';
}

// ─── Boot ───
init();
