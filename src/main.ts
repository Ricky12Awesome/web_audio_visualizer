import { Application, Graphics } from "pixi.js";

const app = new Application();
const mount = document.querySelector<HTMLDivElement>("#app");

if (mount === null) {
    throw new Error("Pixi mount element #app was not found");
}

const appMount = mount;

async function start(): Promise<void> {
    await app.init({
        // An array prevents Pixi from falling back to WebGL.
        preference: ["webgpu"],
        resizeTo: window,
        background: 0x181818,
        antialias: true,
    });

    appMount.appendChild(app.canvas);

    const marker = new Graphics()
        .circle(0, 0, 96)
        .fill(0x7c3aed);

    app.stage.addChild(marker);

    const centerMarker = (): void => {
        marker.position.set(app.screen.width / 2, app.screen.height / 2);
    };

    centerMarker();
    window.addEventListener("resize", centerMarker);
}

void start().catch((error: unknown) => {
    console.error("Unable to initialize Pixi with WebGPU", error);
});
