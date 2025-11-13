# Apicentric Web UI

This is the Next.js frontend for the Apicentric API simulator.

## Frontend Development Quick Start

This project is configured to allow frontend development **without needing to install Rust or compile the backend manually**.

### Prerequisites

- [Node.js and npm](https://nodejs.org/en/download/)

### Installation

Navigate to this directory (`webui/`) and install the required npm packages:
```bash
npm install
```

### Running the Development Server

Once the dependencies are installed, you can start the full-stack application with a single command from this directory (`webui/`):

```bash
npm run dev
```

This command automates the entire backend setup process for you:
1.  **It checks for a local backend binary** in the `webui/backend/` directory.
2.  **If the binary is not found**, it automatically downloads the latest pre-compiled release from GitHub and saves it in `webui/backend/`.
3.  **It runs the backend API server** in the background.
4.  **It starts the Next.js development server** for the frontend.

You can then access the WebUI by opening your browser to [http://localhost:9002](http://localhost:9002).

The `webui/backend/` directory is automatically created and ignored by Git, so you don't have to worry about committing the downloaded binary.
