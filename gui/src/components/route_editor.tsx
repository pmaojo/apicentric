import React, { useEffect } from "react";

export const RouteEditor: React.FC = () => {
  const params = new URLSearchParams(window.location.search);
  const service = params.get("service");
  const endpoint = params.get("endpoint");

  useEffect(() => {
    if (service && endpoint) {
      // Placeholder for highlighting logic
      console.log(`Highlight service ${service} endpoint ${endpoint}`);
    }
  }, [service, endpoint]);

  return (
    <div>
      <h2>Route Editor</h2>
      <p>Service: {service}</p>
      <p>Endpoint: {endpoint}</p>
    </div>
  );
};

export default RouteEditor;
