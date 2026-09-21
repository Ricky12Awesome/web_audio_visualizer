# Web Audio Visualizer

Minimal PixiJS WebGPU setup.

## Development

```sh
npm install
npm run dev
```

The renderer is configured with `preference: ["webgpu"]`, so it does not fall
back to WebGL. Use a browser with WebGPU support.
