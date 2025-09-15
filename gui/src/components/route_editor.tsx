import React, { useEffect, useState } from "react";
import Editor from "@monaco-editor/react";
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

const HTTP_METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE"];

const validatePath = (value: string): string | null => {
  if (!value.trim()) return "Path is required";
  if (!value.startsWith("/")) return "Path must start with /";
  return null;
};

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
}> = ({ scenario, onChange }) => {
  const update = (key: keyof ResponseScenario, value: any) =>
    onChange({ ...scenario, [key]: value });

  return (
    <div style={{ border: "1px solid #ccc", padding: 8, marginBottom: 8 }}>
      <div style={{ display: "flex", alignItems: "center" }}>
        <label htmlFor="status">Status:</label>
        <input
          id="status"
          type="number"
          value={scenario.status}
          onChange={(e) => update("status", Number(e.target.value))}
          style={{ marginLeft: 4, marginRight: 8 }}
        />
      </div>
      <div>
        <label>Body:</label>
        <Editor
          height="120px"
          language="json"
          value={scenario.body}
          onChange={(v) => update("body", v || "")}
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
  const [active, setActive] = useState(0);
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

  const handlePathChange = (value: string) => {
    setPath(value);
    const err = validatePath(value);
    setErrors((prev) => {
      const { path: _, ...rest } = prev;
      return err ? { ...rest, path: err } : rest;
    });
  };

  const validate = () => {
    const err: Record<string, string> = {};
    if (!method.trim()) err.method = "Method is required";
    const pathErr = validatePath(path);
    if (pathErr) err.path = pathErr;
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

  const addResponse = () => {
    const next = [...responses, emptyResponse()];
    setResponses(next);
    setActive(next.length - 1);
  };
  const removeResponse = (idx: number) => {
    const next = responses.filter((_, i) => i !== idx);
    setResponses(next);
    setActive((a) => (a >= next.length ? next.length - 1 : a));
  };
  const moveResponse = (idx: number, dir: number) => {
    const next = [...responses];
    const [item] = next.splice(idx, 1);
    const newIdx = idx + dir;
    next.splice(newIdx, 0, item);
    setResponses(next);
    setActive(newIdx);
  };

  return (
    <div style={{ padding: "1rem" }}>
      <h2>Route Editor</h2>
      {errors.load && (
        <div style={{ color: "red" }}>Failed to load: {errors.load}</div>
      )}
      <div>
        <label htmlFor="method">Method:</label>
        <select
          id="method"
          value={method}
          onChange={(e) => setMethod(e.target.value)}
          style={{ marginLeft: 4 }}
        >
          {HTTP_METHODS.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>
        {errors.method && (
          <span style={{ color: "red", marginLeft: 4 }}>{errors.method}</span>
        )}
      </div>
      <div>
        <label htmlFor="path">Path:</label>
        <input
          id="path"
          value={path}
          onChange={(e) => handlePathChange(e.target.value)}
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
        <div role="tablist" style={{ display: "flex", gap: 4, marginBottom: 8 }}>
          {responses.map((_, i) => (
            <div
              key={i}
              style={{ display: "flex", alignItems: "center", gap: 2 }}
            >
              <button
                role="tab"
                aria-selected={i === active}
                onClick={() => setActive(i)}
              >
                Scenario {i + 1}
              </button>
              <button onClick={() => moveResponse(i, -1)} disabled={i === 0}>
                ◀
              </button>
              <button
                onClick={() => moveResponse(i, 1)}
                disabled={i === responses.length - 1}
              >
                ▶
              </button>
              <button onClick={() => removeResponse(i)}>Delete</button>
            </div>
          ))}
          <button onClick={addResponse}>Add Scenario</button>
        </div>
        {responses[active] && (
          <ResponseEditor
            scenario={responses[active]}
            onChange={(nr) => {
              const next = [...responses];
              next[active] = nr;
              setResponses(next);
            }}
          />
        )}
      </div>
      <div style={{ marginTop: 8 }}>
        <button onClick={onSave}>Save</button>
      </div>
    </div>
  );
};

export default RouteEditor;

