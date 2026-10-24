// Inspects the running app through the Chrome DevTools Protocol (WebView2 started
// with tools/dev-debug.mjs). Development only.
//
//   node tools/cdp.mjs targets
//   node tools/cdp.mjs shot <url-substring> <out.png>     screenshot of a window
//   node tools/cdp.mjs eval <url-substring> "<js>"        evaluate (awaits promises)
//   node tools/cdp.mjs console <url-substring> [seconds]  print console messages and errors
import { writeFileSync } from "node:fs";

const port = process.env.CDP_PORT ?? "9222";
const [cmd, match = "", arg, arg2] = process.argv.slice(2);

const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
const pages = targets.filter((t) => t.type === "page");
if (cmd === "targets") {
  for (const t of pages) console.log(`${t.id}  ${t.title}  ${t.url}`);
  process.exit(0);
}
// "overlay" matches the overlay window; anything else the first page whose URL contains it.
const target =
  pages.find((t) => (match === "main" ? !t.url.includes("/overlay") : t.url.includes(match))) ??
  (match === "main" ? pages[0] : undefined);
if (!target) {
  console.error(`no page matching "${match}"`, pages.map((p) => p.url));
  process.exit(1);
}

const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((ok, ko) => ((ws.onopen = ok), (ws.onerror = ko)));
let next = 0;
const pending = new Map();
const events = [];
ws.onmessage = (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id !== undefined && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  } else if (msg.method) events.push(msg);
};
const send = (method, params = {}) =>
  new Promise((ok) => {
    const id = ++next;
    pending.set(id, ok);
    ws.send(JSON.stringify({ id, method, params }));
  });

if (cmd === "shot") {
  const r = await send("Page.captureScreenshot", { format: "png" });
  if (!r.result) throw new Error(JSON.stringify(r.error));
  writeFileSync(arg, Buffer.from(r.result.data, "base64"));
  console.log("saved", arg);
} else if (cmd === "eval") {
  const r = await send("Runtime.evaluate", { expression: arg, awaitPromise: true, returnByValue: true });
  console.log(JSON.stringify(r.result?.result?.value ?? r.result?.exceptionDetails ?? r, null, 2));
} else if (cmd === "console") {
  await send("Runtime.enable");
  await send("Log.enable");
  await new Promise((ok) => setTimeout(ok, Number(arg ?? 3) * 1000));
  for (const e of events) {
    if (e.method === "Runtime.consoleAPICalled") console.log(`[${e.params.type}]`, e.params.args.map((a) => a.value ?? a.description).join(" "));
    if (e.method === "Runtime.exceptionThrown") console.log("[exception]", e.params.exceptionDetails.exception?.description ?? e.params.exceptionDetails.text);
    if (e.method === "Log.entryAdded") console.log(`[${e.params.entry.level}]`, e.params.entry.text);
  }
}
ws.close();
