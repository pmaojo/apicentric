import React from "react";

interface EndpointSummary {
  method: string;
  path: string;
}

interface SidebarProps {
  servicePath: string;
  endpoints: EndpointSummary[];
  onAdd: () => void;
}

const Sidebar: React.FC<SidebarProps> = ({ servicePath, endpoints, onAdd }) => (
  <div>
    <h3>Endpoints</h3>
    <ul>
      {endpoints.map((ep, i) => (
        <li key={i}>
          <a href={`/route_editor?service=${servicePath}&endpoint=${i}`}>
            {ep.method || "METHOD"} {ep.path || "/path"}
          </a>
        </li>
      ))}
    </ul>
    <button onClick={onAdd}>Add Endpoint</button>
  </div>
);

export default Sidebar;
