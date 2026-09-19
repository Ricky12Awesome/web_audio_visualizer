import { spawn, type ChildProcess } from "node:child_process";
import { resolve } from "node:path";
import type { Plugin, ViteDevServer } from "vite";
import { defineConfig } from "vite";

function rustWasmHotReload(): Plugin {
  let buildProcess: ChildProcess | undefined;
  let rebuildQueued = false;
  let stopped = false;

  const rebuild = (server: ViteDevServer): void => {
    if (buildProcess) {
      rebuildQueued = true;
      return;
    }

    console.log("[rust] rebuilding WASM...");
    const npm = process.platform === "win32" ? "npm.cmd" : "npm";
    buildProcess = spawn(npm, ["run", "wasm"], {
      cwd: process.cwd(),
      stdio: "inherit",
    });

    buildProcess.once("close", (code) => {
      buildProcess = undefined;
      if (code === 0) {
        console.log("[rust] WASM rebuilt; reloading browser");
        server.ws.send({ type: "full-reload" });
      } else {
        console.error(`[rust] WASM build failed with exit code ${code ?? "unknown"}`);
      }

      if (rebuildQueued && !stopped) {
        rebuildQueued = false;
        rebuild(server);
      }
    });
  };

  return {
    name: "rust-wasm-hot-reload",
    configureServer(server) {
      const root = server.config.root;
      const rustPaths = [
        resolve(root, "src/**/*.rs"),
        resolve(root, ".cargo/**/*.toml"),
        resolve(root, "Cargo.toml"),
        resolve(root, "Cargo.lock"),
      ];
      server.watcher.add(rustPaths);

      const onChange = (file: string): void => {
        const isRustChange =
          file.endsWith(".rs") ||
          file.endsWith("Cargo.toml") ||
          file.endsWith("Cargo.lock");
        if (isRustChange) rebuild(server);
      };

      server.watcher.on("change", onChange);
      server.httpServer?.once("close", () => {
        stopped = true;
        server.watcher.off("change", onChange);
        buildProcess?.kill();
      });
    },
  };
}

export default defineConfig({
  plugins: [rustWasmHotReload()],
});
