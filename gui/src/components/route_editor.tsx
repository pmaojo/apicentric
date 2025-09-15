import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { load as loadYaml, dump as dumpYaml } from "js-yaml";

/** Port for loading and saving services */
interface ServicePort {
  load(path: string): Promise<any>;
  save(path: string, data: any): Promise<void>;
}

const servicePort: ServicePort = {
  load: async (path) => {
    const yamlStr = await invoke<string>("load_service", { path });
    return loadYaml(yamlStr);
  },
  save: async (path, data) => {
    const yamlStr = dumpYaml(data);
    await invoke("save_service", { path, yaml: yamlStr });
  },
};

interface Header {
  key: string;
  value: string;
}

interface ResponseScenario {
  status: number;
  headers: Header[];
  body: string;
}

const emptyHeader = (): Header => ({ key: "", value: "" });
const emptyResponse = (): ResponseScenario => ({
  status: 200,
  headers: [],
  body: "",
});

const HeadersEditor: React.FC<{
  headers: Header[];
  onChange: (h: Header[]) => void;
}> = ({ headers, onChange }) => {
  const update = (index: number, key: keyof Header, value: string) => {
    const next = [...headers];
    next[index] = { ...next[index], [key]: value };
    onChange(next);
  };
  const add = () => onChange([...headers, emptyHeader()]);
  const remove = (idx: number) => onChange(headers.filter((_, i) => i !== idx));

  return (
    <div>
      {headers.map((h, i) => (
        <div key={i} style={{ display: "flex", marginBottom: 4 }}>
          <input
            placeholder="Header"
            value={h.key}
            onChange={(e) => update(i, "key", e.target.value)}
            style={{ marginRight: 4 }}
          />
          <input
            placeholder="Value"
            value={h.value}
            onChange={(e) => update(i, "value", e.target.value)}
            style={{ marginRight: 4 }}
          />
          <button onClick={() => remove(i)}>X</button>
        </div>
      ))}
      <button onClick={add}>Add Header</button>
    </div>
  );
};

const ResponseEditor: React.FC<{
  scenario: ResponseScenario;
  onChange: (s: ResponseScenario) => void;
  onRemove: () => void;
}> = ({ scenario, onChange, onRemove }) => {
  const update = (key: keyof ResponseScenario, value: any) =>
    onChange({ ...scenario, [key]: value });

  return (
    <div style={{ border: "1px solid #ccc", padding: 8, marginBottom: 8 }}>
      <div style={{ display: "flex", alignItems: "center" }}>
        <label>Status:</label>
        <input
          type="number"
          value={scenario.status}
          onChange={(e) => update("status", Number(e.target.value))}
          style={{ marginLeft: 4, marginRight: 8 }}
        />
        <button onClick={onRemove}>Remove</button>
      </div>
      <div>
        <label>Body:</label>
        <textarea
          value={scenario.body}
          onChange={(e) => update("body", e.target.value)}
          style={{ width: "100%", height: 80 }}
        />
      </div>
      <div>
        <label>Headers:</label>
        <HeadersEditor
          headers={scenario.headers}
          onChange={(h) => update("headers", h)}
        />
      </div>
    </div>
  );
};

export const RouteEditor: React.FC = () => {
  const params = new URLSearchParams(window.location.search);
  const service = params.get("service");
  const endpointIdx = parseInt(params.get("endpoint") || "0", 10);

  const [method, setMethod] = useState("");
  const [path, setPath] = useState("");
  const [headers, setHeaders] = useState<Header[]>([]);
  const [payload, setPayload] = useState("");
  const [responses, setResponses] = useState<ResponseScenario[]>([]);
  const [strategy, setStrategy] = useState<"sequential" | "random">(
    "sequential"
  );
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [serviceDef, setServiceDef] = useState<any | null>(null);

  useEffect(() => {
    if (!service) return;
    servicePort
      .load(service)
      .then((def) => {
        setServiceDef(def);
        const ep = def.endpoints?.[endpointIdx];
        if (ep) {
          setMethod(ep.method || "");
          setPath(ep.path || "");
          setHeaders(
            Object.entries(ep.headers || {}).map(([key, value]) => ({
              key,
              value: String(value),
            }))
          );
          setPayload(ep.payload || "");
          const resp = ep.responses || { strategy: "sequential", list: [] };
          setStrategy(resp.strategy || "sequential");
          setResponses(
            (resp.list || []).map((r: any) => ({
              status: r.status,
              headers: Object.entries(r.headers || {}).map(([k, v]) => ({
                key: k,
                value: String(v),
              })),
              body: r.body || "",
            }))
          );
        }
      })
      .catch((e) => {
        setErrors({ load: String(e) });
      });
  }, [service, endpointIdx]);

  const validate = () => {
    const err: Record<string, string> = {};
    if (!method.trim()) err.method = "Method is required";
    if (!path.trim()) err.path = "Path is required";
    setErrors(err);
    return Object.keys(err).length === 0;
  };

  const onSave = async () => {
    if (!service || !serviceDef) return;
    if (!validate()) return;
    const ep = {
      method,
      path,
      headers: Object.fromEntries(
        headers.filter((h) => h.key).map((h) => [h.key, h.value])
      ),
      payload,
      responses: {
        strategy,
        list: responses.map((r) => ({
          status: r.status,
          headers: Object.fromEntries(
            r.headers.filter((h) => h.key).map((h) => [h.key, h.value])
          ),
          body: r.body,
        })),
      },
    };
    const def = { ...serviceDef };
    if (!def.endpoints) def.endpoints = [];
    def.endpoints[endpointIdx] = ep;
    await servicePort.save(service, def);
    alert("Saved");
  };

  const addResponse = () => setResponses([...responses, emptyResponse()]);
  const removeResponse = (idx: number) =>
    setResponses(responses.filter((_, i) => i !== idx));

  return (
    <div style={{ padding: "1rem" }}>
      <h2>Route Editor</h2>
      {errors.load && (
        <div style={{ color: "red" }}>Failed to load: {errors.load}</div>
      )}
      <div>
        <label htmlFor="method">Method:</label>
        <input
          id="method"
          value={method}
          onChange={(e) => setMethod(e.target.value)}
          style={{ marginLeft: 4 }}
        />
        {errors.method && (
          <span style={{ color: "red", marginLeft: 4 }}>{errors.method}</span>
        )}
      </div>
      <div>
        <label htmlFor="path">Path:</label>
        <input
          id="path"
          value={path}
          onChange={(e) => setPath(e.target.value)}
          style={{ marginLeft: 4 }}
        />
        {errors.path && (
          <span style={{ color: "red", marginLeft: 4 }}>{errors.path}</span>
        )}
      </div>
      <div>
        <label>Request Headers:</label>
        <HeadersEditor headers={headers} onChange={setHeaders} />
      </div>
      <div>
        <label>Payload:</label>
        <textarea
          value={payload}
          onChange={(e) => setPayload(e.target.value)}
          style={{ width: "100%", height: 100 }}
        />
      </div>
      <div>
        <label>Response Strategy:</label>
        <select
          value={strategy}
          onChange={(e) => setStrategy(e.target.value as any)}
          style={{ marginLeft: 4 }}
        >
          <option value="sequential">Sequential</option>
          <option value="random">Random</option>
        </select>
      </div>
      <div>
        <h3>Responses</h3>
        {responses.map((r, i) => (
          <ResponseEditor
            key={i}
            scenario={r}
            onChange={(nr) => {
              const next = [...responses];
              next[i] = nr;
              setResponses(next);
            }}
            onRemove={() => removeResponse(i)}
          />
        ))}
        <button onClick={addResponse}>Add Response</button>
      </div>
      <div style={{ marginTop: 8 }}>
        <button onClick={onSave}>Save</button>
      </div>
    </div>
  );
};

export default RouteEditor;

