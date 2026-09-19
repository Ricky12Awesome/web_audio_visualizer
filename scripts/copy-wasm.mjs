import { copyFile, mkdir } from "node:fs/promises";
import { resolve } from "node:path";

const projectRoot = resolve(import.meta.dirname, "..");
const source = resolve(
  projectRoot,
  "target/wasm32-unknown-unknown/release/web_audio_visualizer.wasm",
);
const destinationDirectory = resolve(projectRoot, "public/wasm");
const destination = resolve(destinationDirectory, "web_audio_visualizer.wasm");

await mkdir(destinationDirectory, { recursive: true });
await copyFile(source, destination);
console.log(`Copied Rust WASM to ${destination}`);
