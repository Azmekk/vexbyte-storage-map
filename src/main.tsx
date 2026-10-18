import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import { getPlatformInfo } from "./lib/backend/platform";
import { initializeCurrentPlatform } from "./lib/platform/currentPlatform";
import "./index.css";

async function loadPlatformBeforeFirstRender(): Promise<void> {
  try {
    initializeCurrentPlatform(await getPlatformInfo());
  } catch (error) {
    console.error("Could not read the platform; using the Windows wording.", error);
  }
}

const rootElement = document.getElementById("root");

if (rootElement === null) {
  throw new Error("The #root element is missing from index.html.");
}

void loadPlatformBeforeFirstRender().then(() => {
  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
