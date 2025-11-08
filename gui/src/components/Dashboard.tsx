import React from "react";
import { Link } from "react-router-dom";
import LogsView from "./logs_view";

export const Dashboard: React.FC = () => {
  return (
    <div style={{ padding: "1rem", fontFamily: "sans-serif" }}>
      <h1>Welcome to Apicentric Cloud</h1>
      <p>
        This is the central hub for managing and simulating your API services.
      </p>
      <p>
        Use the navigation on the left to get started:
      </p>
      <ul>
        <li>
          <Link to="/services">Service Manager</Link>: Load, view, and save your
          service definitions.
        </li>
        <li>
          <Link to="/simulator">Simulator</Link>: Start, stop, and monitor your
          mock services.
        </li>
      </ul>

      <section id="logs">
        <h2>Recent Global Logs</h2>
        <LogsView serviceId="global" />
      </section>
    </div>
  );
};
