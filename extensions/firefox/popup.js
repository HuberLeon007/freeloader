// SPDX-License-Identifier: GPL-3.0-or-later
const HOST = "io.freeloader.host";
const status = document.getElementById("status");
const list = document.getElementById("links");
document.getElementById("scan").addEventListener("click", async () => {
  list.replaceChildren();
  const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id) { status.textContent = "No active page available."; return; }
  const results = await browser.tabs.executeScript(tab.id, { code: "Array.from(document.querySelectorAll('a[href],img[src],video[src],audio[src]')).map((node) => node.getAttribute('href') || node.getAttribute('src')).filter((value) => typeof value === 'string' && /^https?:\\/\\//i.test(value))" });
  const urls = [...new Set(results[0] || [])];
  if (urls.length === 0) { status.textContent = "No direct HTTP(S) links found."; return; }
  status.textContent = `${urls.length} candidate(s) found. Select what to send.`;
  for (const url of urls.slice(0, 50)) {
    const item = document.createElement("li"); const label = document.createElement("label"); const checkbox = document.createElement("input"); checkbox.type = "checkbox"; checkbox.checked = true;
    checkbox.addEventListener("change", () => { if (checkbox.checked) void browser.runtime.sendNativeMessage(HOST, { version: 1, type: "capture_download", payload: { url, suggestedFilename: null, referrer: tab.url || null, contentType: null, cookiesIncluded: false } }); });
    label.append(checkbox, document.createTextNode(url)); item.append(label); list.append(item); checkbox.dispatchEvent(new Event("change"));
  }
});
