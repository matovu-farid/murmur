import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Routes, Route } from "react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./lib/queryClient";
import { Overlay } from "./pages/Overlay";
import { Settings } from "./pages/Settings";
import { History } from "./pages/History";
import { Onboarding } from "./pages/Onboarding";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route path="/" element={<Overlay />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/history" element={<History />} />
          <Route path="/onboarding" element={<Onboarding />} />
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  </React.StrictMode>
);
