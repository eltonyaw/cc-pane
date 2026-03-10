const fs = require("fs");
const path = require("path");
const d = path.join("src-tauri", "binaries");
fs.mkdirSync(d, { recursive: true });
fs.copyFileSync(
  path.join("target", "release", "cc-panes-hook.exe"),
  path.join(d, "cc-panes-hook.exe")
);
