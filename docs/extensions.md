# Browser extensions

Freeloader does not use the Chrome Web Store. Chromium users download the release ZIP from GitHub Releases, extract it to a stable directory, enable Developer Mode on the browser's Extensions page, and choose **Load unpacked**. The release process must replace the `REPLACE_WITH_RELEASE_PUBLIC_KEY` manifest value with a real release key before publishing; the resulting extension ID must match the exact `allowed_origins` value in the Native Messaging host manifest. Wildcards are forbidden.

Firefox is distributed through Firefox Add-ons and can also be loaded temporarily from the extracted package. Microsoft Edge may use Edge Add-ons. Store URLs are configuration values, not hardcoded into browser-detection logic.

The extension only sends explicit HTTP(S) links selected through the context menu or an explicit page scan. It does not read cookies, authorization headers, browser history, bookmarks or background network requests.
