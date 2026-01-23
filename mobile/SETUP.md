# Apicentric Mobile

A Flutter-based mobile application for running Apicentric API simulators.

## Prerequisites

- **Flutter SDK**: [Install Flutter](https://docs.flutter.dev/get-started/install)
- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Android NDK**: Required for compiling Rust code for Android.
- **iOS Development Tools**: Xcode (macOS only) for iOS.

## Setup

1.  **Install Flutter Rust Bridge Codegen:**

    ```bash
    cargo install flutter_rust_bridge_codegen
    ```

2.  **Generate Bindings:**

    Run this command from the `mobile` directory to generate the Dart-Rust bridge code. **You must do this before running the app**, as the current `lib/src/rust/api.dart` is a placeholder.

    ```bash
    flutter_rust_bridge_codegen generate --rust-root native --rust-input crate::api --dart-output lib/src/rust/api.dart --dart-entrypoint-class-name RustLib
    ```

3.  **Install Dart Dependencies:**

    ```bash
    flutter pub get
    ```

## Running the App

### Android

Ensure you have an Android device connected or an emulator running.

```bash
flutter run
```

### iOS

Ensure you have an iOS device connected or a simulator running.

```bash
flutter run
```

## Features

- **Active Simulations:** View and control running API services.
- **Library:** Manage your simulation configuration files (YAML).
- **Create Wizard:**
  - **Manual:** Create simple services via form.
  - **AI:** Generate services using OpenAI prompts.
- **Background Execution:** Simulator runs in a foreground service to ensure persistence.
