# Request Recording & Auto-Mocking Guide

Apicentric can observe real HTTP traffic and turn it into runnable mock services. This guide
shows how to capture requests against an existing API, generate YAML service definitions, and keep
expanding your mocks automatically with `record_unknown`.

## Prerequisites

- Apicentric CLI installed
- Access to a real API you can call (staging, local, or production clone)
- Basic familiarity with sending HTTP requests (curl, Postman, browser, etc.)

## Option 1: Capture Traffic with the Recording Proxy

Use the built-in recording proxy when you want to explore an API quickly and generate a fresh
service definition from live traffic.

1. Choose an output directory for generated services (it will be created if missing).
2. Run the recorder, pointing it at the upstream API:

   ```bash
   apicentric simulator record \
     --output recordings \
     --url https://api.example.com
   ```

   - `--output` controls where the generated YAML will be written (`services` by default).
   - `--url` overrides the base URL from `apicentric.json`; omit it to reuse the configured
     `base_url`.

   The recorder listens on the first port in your simulator `port_range` (9000 by default) and
   forwards every request to the upstream target.

3. Send requests to the recorder instead of the real API. For example:

   ```bash
   curl http://localhost:9000/v1/users
   ```

   Each request/response pair is proxied to the target API and stored in memory. When you stop the
   recorder with `Ctrl+C`, Apicentric writes a `recorded_service.yaml` file into the chosen
   directory.

4. Inspect the generated YAML and either keep it as-is or merge it into an existing service.

## Option 2: Auto-Generate Endpoints While Mocking

`record_unknown` lets a running simulator proxy unexpected routes to a real API while adding the
observed interactions to your existing service definition. This is ideal when you already have
partial mocks and want to grow them incrementally.

1. Configure your service to forward traffic that is not mocked yet by setting
   `proxy_base_url` and `record_unknown` in the `server` section:

   ```yaml
   server:
     port: 9000
     base_path: /api
     proxy_base_url: https://api.example.com
     record_unknown: true
   ```

2. Start the simulator as usual:

   ```bash
   apicentric simulator start --services-dir services
   ```

3. Exercise your application or send HTTP requests against the simulator. Whenever a request does
   not match a defined endpoint, Apicentric proxies the call to `proxy_base_url`, captures the
   response, normalizes the path parameters, and appends a new endpoint to the service YAML.

4. Restart the simulator (or reload the file) to pick up the newly recorded endpoints. Review and
   refine the generated handlers just like any other mock.

## Tips for High-Quality Recordings

- **Use dedicated environments.** Recording copies real payloads verbatim, so prefer staging data or
  sanitized fixtures.
- **Group related traffic.** Running the recorder per feature or scenario keeps generated YAML
  focused and easier to maintain.
- **Normalize sensitive headers.** Remove or redact secrets (tokens, cookies) from the generated
  files before committing them.
- **Pair with contract tests.** Once endpoints exist, add contract tests to detect drift between the
  mock and the upstream API.

With recording and `record_unknown` enabled, Apicentric bridges the gap between real systems and
fast, reliable mocks.
