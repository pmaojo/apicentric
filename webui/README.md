# Apicentric Web UI

This is the Next.js frontend for the Apicentric API simulator.

## Getting Started

To run the full-stack development environment, which includes the Rust backend API and the Next.js frontend, follow these steps.

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- [Node.js and npm](https://nodejs.org/en/download/)

### Installation

1.  **Install Rust dependencies.**
    The backend is a Rust application. The `dev` command will handle building it for you, but you need to have the Rust toolchain installed.

2.  **Install Node.js dependencies.**
    Navigate to this directory (`webui/`) and install the required npm packages:
    ```bash
    npm install
    ```

### Running the Development Server

Once the dependencies are installed, you can start the full-stack application with a single command from this directory (`webui/`):

```bash
npm run dev
```

This command will:
1.  **Build and run the Apicentric Cloud API server**, which is the dedicated backend for the WebUI.
2.  **Start the Next.js development server** for the frontend.

You can then access the WebUI by opening your browser to [http://localhost:9002](http://localhost:9002).
