// Starts `tauri dev` with WebView2's DevTools port open, so tools/cdp.mjs can
// inspect the running app (screenshots, console, evaluate). Development only.
import { spawn } from "node:child_process";

const port = process.env.CDP_PORT ?? "9222";
const child = spawn("npm", ["run", "tauri", "dev"], {
  stdio: "inherit",
  shell: true,
  env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}` },
});
child.on("exit", (code) => process.exit(code ?? 0));
