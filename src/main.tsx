import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// Apply the OS color scheme before first paint to avoid a light flash. Once the
// saved theme setting loads, useTorApp reconciles this (auto/light/dark).
if (
  window.matchMedia &&
  window.matchMedia("(prefers-color-scheme: dark)").matches
) {
  document.documentElement.classList.add("dark");
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
