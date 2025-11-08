import React from "react";
import { Routes, Route, Link } from "react-router-dom";
import Dashboard from "./components/Dashboard";
import RouteEditor from "./components/route_editor";
import ServiceManager from "./components/ServiceManager";
import SimulatorControls from "./components/SimulatorControls";

const Sidebar: React.FC = () => (
  <div style={{ width: "200px", background: "#f0f0f0", padding: "1rem" }}>
    <nav>
      <ul>
        <li>
          <Link to="/">Dashboard</Link>
        </li>
        <li>
          <Link to="/services">Service Manager</Link>
        </li>
        <li>
          <Link to="/simulator">Simulator</Link>
        </li>
        <li>
          <Link to="/route_editor">Route Editor</Link>
        </li>
      </ul>
    </nav>
  </div>
);

const App: React.FC = () => {
  return (
    <div style={{ display: "flex", height: "100vh" }}>
      <Sidebar />
      <div style={{ flex: 1, padding: "1rem", overflow: "auto" }}>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/services" element={<ServiceManager />} />
          <Route path="/simulator" element={<SimulatorControls />} />
          <Route path="/route_editor" element={<RouteEditor />} />
        </Routes>
      </div>
    </div>
  );
};

export default App;
