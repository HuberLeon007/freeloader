// SPDX-License-Identifier: GPL-3.0-or-later
const HOST = "io.freeloader.host";

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({ id: "freeloader-link", title: "Download with Freeloader", contexts: ["link", "image", "video", "audio"] });
});

chrome.contextMenus.onClicked.addListener((info) => {
  const candidate = info.linkUrl || info.srcUrl;
  if (!candidate || !/^https?:\/\//i.test(candidate)) return;
  chrome.runtime.sendNativeMessage(HOST, { version: 1, type: "capture_download", payload: { url: candidate, suggestedFilename: null, referrer: info.pageUrl || null, contentType: null, cookiesIncluded: false } }, () => {
    void chrome.runtime.lastError;
  });
});
