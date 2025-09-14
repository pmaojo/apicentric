import React from "react";
import { Routes, Route } from "react-router-dom";
import Dashboard from "./components/Dashboard";
import RouteEditor from "./components/route_editor";

const App: React.FC = () => {
  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/route_editor" element={<RouteEditor />} />
    </Routes>
  );
};

export default App;
