import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import { ServicePortProvider } from "./ports/ServicePort";
import { tauriServicePort } from "./adapters/TauriServicePort";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <ServicePortProvider port={tauriServicePort}>
        <App />
      </ServicePortProvider>
    </BrowserRouter>
  </React.StrictMode>
);
