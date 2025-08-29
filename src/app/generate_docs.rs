use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::errors::{PulseError, PulseResult};

/// Genera la documentación TypeScript utilizando TypeDoc.
pub fn generate_docs(
    project_root: &Path,
    output: &str,
    serve: bool,
    watch: bool,
    tui_mode: bool,
) -> PulseResult<()> {
    // Verificar que TypeDoc esté disponible
    let typedoc_check = Command::new("npx")
        .args(["typedoc", "--version"])
        .current_dir(project_root)
        .output();

    if typedoc_check.is_err() {
        return Err(PulseError::config_error(
            "TypeDoc no está instalado",
            Some("Ejecuta: npm install --save-dev typedoc"),
        ));
    }

    // Verificar que existe tsconfig.json
    let tsconfig_path = project_root.join("tsconfig.json");
    if !tsconfig_path.exists() {
        return Err(PulseError::config_error(
            "No se encontró tsconfig.json",
            Some("Asegúrate de estar en la raíz del proyecto TypeScript"),
        ));
    }

    // Verificar que existe typedoc.json
    let typedoc_config = project_root.join("typedoc.json");
    if !typedoc_config.exists() {
        println!("⚠️ No se encontró typedoc.json, usando configuración por defecto");
    }

    let output_dir = project_root.join(output);

    if watch {
        // Modo watch
        if tui_mode {
            println!("🔄 Iniciando documentación en modo watch...");
        }

        let mut cmd = Command::new("npx");
        cmd.args(["typedoc", "--watch"]).current_dir(project_root);

        if typedoc_config.exists() {
            cmd.args(["--options", "typedoc.json"]);
        } else {
            cmd.args(["--out", output])
                .args(["--entryPoints", "app/"])
                .args(["--exclude", "**/*.test.ts", "**/*.spec.ts"])
                .args(["--excludePrivate"])
                .args(["--includeVersion"]);
        }

        let mut child = cmd.spawn().map_err(|e| {
            PulseError::runtime_error(
                format!("Error al iniciar TypeDoc watch: {}", e),
                Some("Verifica que TypeDoc esté instalado correctamente"),
            )
        })?;

        if tui_mode {
            // En modo TUI, ejecutar en background
            thread::spawn(move || {
                let _ = child.wait();
            });
            thread::sleep(Duration::from_millis(1000));
            println!("✅ Documentación watch iniciada en background");
        } else {
            // En modo CLI, esperar
            child.wait().map_err(|e| {
                PulseError::runtime_error(format!("Error en TypeDoc watch: {}", e), None::<String>)
            })?;
        }
    } else {
        // Generación única
        if tui_mode {
            println!("📚 Generando documentación...");
        } else {
            println!("📚 Generando documentación TypeScript...");
            println!("📁 Proyecto: {}", project_root.display());
            println!("📁 Salida: {}", output_dir.display());
        }

        let mut cmd = Command::new("npx");
        cmd.args(["typedoc"]).current_dir(project_root);

        if typedoc_config.exists() {
            cmd.args(["--options", "typedoc.json"]);
        } else {
            cmd.args(["--out", output])
                .args(["--entryPoints", "app/"])
                .args(["--exclude", "**/*.test.ts", "**/*.spec.ts"])
                .args(["--excludePrivate"])
                .args(["--includeVersion"]);
        }

        let output_result = cmd.output().map_err(|e| {
            PulseError::runtime_error(
                format!("Error al ejecutar TypeDoc: {}", e),
                Some("Verifica que TypeDoc esté instalado correctamente"),
            )
        })?;

        if !output_result.status.success() {
            let error_msg = String::from_utf8_lossy(&output_result.stderr);
            return Err(PulseError::runtime_error(
                format!("TypeDoc falló: {}", error_msg),
                Some("Revisa la configuración de TypeDoc y los archivos TypeScript"),
            ));
        }

        if tui_mode {
            println!("✅ Documentación generada exitosamente");
        } else {
            println!(
                "✅ Documentación generada exitosamente en: {}",
                output_dir.display()
            );
        }
    }

    if serve {
        // Servir documentación
        if tui_mode {
            println!("🌐 Iniciando servidor de documentación...");
        } else {
            println!("🌐 Iniciando servidor de documentación en http://localhost:8080");
        }

        let mut cmd = Command::new("npx");
        cmd.args(["http-server", output, "-p", "8080", "-o"])
            .current_dir(project_root);

        if tui_mode {
            // En modo TUI, ejecutar en background
            let mut child = cmd.spawn().map_err(|e| {
                PulseError::runtime_error(
                    format!("Error al iniciar servidor: {}", e),
                    Some("Instala http-server: npm install -g http-server"),
                )
            })?;

            thread::spawn(move || {
                let _ = child.wait();
            });

            thread::sleep(Duration::from_millis(2000));
            println!("✅ Servidor iniciado en http://localhost:8080");
        } else {
            // En modo CLI, ejecutar y esperar
            let mut child = cmd.spawn().map_err(|e| {
                PulseError::runtime_error(
                    format!("Error al iniciar servidor: {}", e),
                    Some("Instala http-server: npm install -g http-server"),
                )
            })?;

            println!("🌐 Servidor ejecutándose... Presiona Ctrl+C para detener");
            child.wait().map_err(|e| {
                PulseError::runtime_error(format!("Error en servidor: {}", e), None::<String>)
            })?;
        }
    }

    Ok(())
}
