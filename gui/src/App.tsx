import React from "react";
import { Routes, Route } from "react-router-dom";
import Dashboard from "./components/Dashboard";
import RouteEditor from "./components/route_editor";
import Sidebar from "./components/Sidebar";

const App: React.FC = () => {
  return (
    <div style={{ display: "flex", height: "100vh" }}>
      <Sidebar />
      <div style={{ flex: 1, overflow: "auto" }}>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/route_editor" element={<RouteEditor />} />
        </Routes>
      </div>
    </div>
  );
};

export default App;
