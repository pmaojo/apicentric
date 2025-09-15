import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { load as loadYaml, dump as dumpYaml } from "js-yaml";
import { useNavigate } from "react-router-dom";

interface EndpointSummary {
  method: string;
  path: string;
}

interface Service {
  path: string;
  name: string;
  endpoints: Endpoint[];
  open?: boolean;
}

interface ListServiceInfo {
  name: string;
}

interface ServicePort {
  list(): Promise<ListServiceInfo[]>;
  load(path: string): Promise<Service>;
  save(path: string, data: Service): Promise<void>;
}

const servicePort: ServicePort = {
  list: async () => {
    const services = await invoke<ListServiceInfo[]>("list_services");
    return services;
  },
  load: async (path) => {
    const yamlStr = await invoke<string>("load_service", { path });
    const data: any = loadYaml(yamlStr) || {};
    return {
      path,
      name: data.name || path,
      endpoints: (data.endpoints || []).map((e: any) => ({
        method: e.method || "GET",
        path: e.path || "/",
      })),
    };
  },
  save: async (path, data) => {
    const yamlStr = dumpYaml({
      name: data.name,
      endpoints: data.endpoints,
    });
    await invoke("save_service", { path, yaml: yamlStr });
  },
};

const Sidebar: React.FC = () => {
  const [services, setServices] = useState<Service[]>([]);
  const navigate = useNavigate();

  const refresh = async () => {
    const list = await servicePort.list();
    const defs = await Promise.all(list.map((s) => servicePort.load(s.name)));
    setServices(defs);
  };

  useEffect(() => {
    refresh();
  }, []);

  const addService = async () => {
    const path = prompt("New service path?", "service.yaml");
    if (!path) return;
    const service: Service = { path, name: path, endpoints: [] };
    await servicePort.save(path, service);
    setServices([...services, service]);
  };

  const renameService = async (svc: Service) => {
    const name = prompt("Rename service", svc.name);
    if (!name) return;
    const updated = { ...svc, name };
    await servicePort.save(svc.path, updated);
    setServices(services.map((s) => (s.path === svc.path ? updated : s)));
  };

  const deleteService = async (svc: Service) => {
    if (!confirm(`Delete service ${svc.name}?`)) return;
    await servicePort.save(svc.path, { ...svc, name: svc.name, endpoints: [] });
    setServices(services.filter((s) => s.path !== svc.path));
  };

  const addEndpoint = async (svc: Service) => {
    const path = prompt("Endpoint path?", "/new");
    if (!path) return;
    const method = prompt("Method?", "GET") || "GET";
    const updated = {
      ...svc,
      endpoints: [...svc.endpoints, { method, path }],
    };
    await servicePort.save(svc.path, updated);
    setServices(services.map((s) => (s.path === svc.path ? updated : s)));
  };

  const deleteEndpoint = async (svc: Service, idx: number) => {
    const updated = {
      ...svc,
      endpoints: svc.endpoints.filter((_, i) => i !== idx),
    };
    await servicePort.save(svc.path, updated);
    setServices(services.map((s) => (s.path === svc.path ? updated : s)));
  };

  const toggleOpen = (svc: Service) => {
    setServices(
      services.map((s) =>
        s.path === svc.path ? { ...s, open: !s.open } : s
      )
    );
  };

  return (
    <div style={{ width: 250, borderRight: "1px solid #ccc", padding: 8 }}>
      <div style={{ display: "flex", justifyContent: "space-between" }}>
        <strong>Services</strong>
        <button onClick={addService}>+</button>
      </div>
      <ul style={{ listStyle: "none", paddingLeft: 0 }}>
        {services.map((svc) => (
          <li key={svc.path}>
            <div style={{ display: "flex", alignItems: "center" }}>
              <button onClick={() => toggleOpen(svc)} style={{ marginRight: 4 }}>
                {svc.open ? "-" : "+"}
              </button>
              <span
                style={{ flex: 1, cursor: "pointer" }}
                onClick={() => toggleOpen(svc)}
              >
                {svc.name}
              </span>
              <button onClick={() => addEndpoint(svc)}>+</button>
              <button onClick={() => renameService(svc)}>R</button>
              <button onClick={() => deleteService(svc)}>X</button>
            </div>
            {svc.open && (
              <ul style={{ listStyle: "none", paddingLeft: 16 }}>
                {svc.endpoints.map((ep, idx) => (
                  <li key={idx} style={{ display: "flex", alignItems: "center" }}>
                    <span
                      style={{ flex: 1, cursor: "pointer" }}
                      onClick={() =>
                        navigate(`/route_editor?service=${svc.path}&endpoint=${idx}`)
                      }
                    >
                      {ep.method} {ep.path}
                    </span>
                    <button onClick={() => deleteEndpoint(svc, idx)}>X</button>
                  </li>
                ))}
              </ul>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
};


export default Sidebar;
