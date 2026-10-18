import "@fontsource-variable/bricolage-grotesque";
import "@fontsource-variable/onest";
import "@fontsource-variable/jetbrains-mono";
import "./styles/global.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

const rootElement = document.getElementById("root");
if (rootElement) {
  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
