import React from "react";
import { Routes, Route } from "react-router-dom";
import SimulatorControls from "./components/SimulatorControls";
import ServiceManager from "./components/ServiceManager";
import RouteEditor from "./components/route_editor";
import LogsView from "./components/logs_view";

const Home: React.FC = () => (
  <div>
    <h1>MockForge GUI</h1>
    <SimulatorControls />
    <ServiceManager />
    <LogsView />
  </div>
);

const App: React.FC = () => {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/route_editor" element={<RouteEditor />} />
    </Routes>
  );
};

export default App;
