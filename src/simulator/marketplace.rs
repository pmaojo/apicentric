use crate::{ApicentricError, ApicentricResult};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::simulator::openapi::from_openapi;
use crate::simulator::ServiceDefinition;


/// Marketplace Item
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketplaceItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String, // "SaaS", "Device", "Template"
    pub logo_url: Option<String>,
    pub definition_url: String,
}

/// Returns the list of available marketplace items.
pub fn get_marketplace_items() -> Vec<MarketplaceItem> {
    vec![
        // SaaS APIs
        MarketplaceItem {
            id: "stripe".to_string(),
            name: "Stripe API".to_string(),
            description: "Mock Stripe API with payments, customers, and subscriptions.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.yaml".to_string(),
        },
        MarketplaceItem {
            id: "slack".to_string(),
            name: "Slack API".to_string(),
            description: "Mock Slack Web API for messaging and channels.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/slackapi/slack-api-specs/master/web-api/slack_web_api.yaml".to_string(),
        },
        MarketplaceItem {
            id: "github".to_string(),
            name: "GitHub API".to_string(),
            description: "Mock GitHub REST API for repositories and issues.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.yaml".to_string(),
        },
        MarketplaceItem {
            id: "openai".to_string(),
            name: "OpenAI API".to_string(),
            description: "Mock OpenAI API for chat completions and embeddings.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml".to_string(),
        },
        MarketplaceItem {
            id: "kubernetes".to_string(),
            name: "Kubernetes API".to_string(),
            description: "Mock Kubernetes API (Core) for pods, services, and deployments.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/kubernetes/kubernetes/master/api/openapi-spec/swagger.json".to_string(),
        },
        MarketplaceItem {
            id: "sendgrid".to_string(),
            name: "SendGrid API".to_string(),
            description: "Mock SendGrid v3 API for email delivery and marketing.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/sendgrid/sendgrid-oai/master/oai_stoplight.json".to_string(),
        },
        MarketplaceItem {
            id: "digitalocean".to_string(),
            name: "DigitalOcean API".to_string(),
            description: "Mock DigitalOcean Public API for droplets and volumes.".to_string(),
            category: "SaaS".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/digitalocean/openapi/main/specification/DigitalOcean-public.yaml".to_string(),
        },

        // Templates
        MarketplaceItem {
            id: "petstore".to_string(),
            name: "PetStore".to_string(),
            description: "Standard Swagger PetStore example API.".to_string(),
            category: "Template".to_string(),
            logo_url: None,
            definition_url: "https://petstore.swagger.io/v2/swagger.json".to_string(),
        },

        // Device APIs
        MarketplaceItem {
            id: "philips-hue".to_string(),
            name: "Philips Hue".to_string(),
            description: "Mock Philips Hue Bridge API for smart lights.".to_string(),
            category: "Device".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/dummy/hue-spec/main/hue.yaml".to_string(), // Placeholder
        },
        MarketplaceItem {
            id: "sonos".to_string(),
            name: "Sonos".to_string(),
            description: "Mock Sonos Control API for smart speakers.".to_string(),
            category: "Device".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/sonos/sonos-developer-portal/master/static/specs/openapi.yaml".to_string(), // Placeholder
        },
        MarketplaceItem {
            id: "acme-sensor".to_string(),
            name: "Acme Smart Sensor".to_string(),
            description: "IoT Sensor API for reading temperature, humidity, and device status.".to_string(),
            category: "Device".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/acme-sensor.yaml".to_string(),
        },

        // IoT Twins
        MarketplaceItem {
            id: "smart-thermostat".to_string(),
            name: "Smart Thermostat".to_string(),
            description: "Digital Twin of a smart thermostat (temperature, humidity). MQTT.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smart-thermostat.yaml".to_string(),
        },
        MarketplaceItem {
            id: "industrial-pump".to_string(),
            name: "Industrial Pump".to_string(),
            description: "Digital Twin of an industrial pump (RPM, flow, temp). Modbus TCP.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/industrial-pump.yaml".to_string(),
        },
        MarketplaceItem {
            id: "solar-inverter".to_string(),
            name: "Solar Inverter".to_string(),
            description: "Digital Twin of a solar inverter (power, voltage). Modbus TCP.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/solar-inverter.yaml".to_string(),
        },
        MarketplaceItem {
            id: "weather-station".to_string(),
            name: "Weather Station".to_string(),
            description: "Digital Twin of a weather station (wind, temp, humidity). MQTT.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/weather-station.yaml".to_string(),
        },
        MarketplaceItem {
            id: "smart-meter".to_string(),
            name: "Smart Meter".to_string(),
            description: "Digital Twin of a smart energy meter. MQTT.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smart-meter.yaml".to_string(),
        },
        // New IoT Twins
        MarketplaceItem {
            id: "victron-energy-system".to_string(),
            name: "Victron Energy System".to_string(),
            description: "Digital Twin of Victron GX device (Batt Voltage, SoC, PV Power). Modbus TCP.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/victron-energy-system.yaml".to_string(),
        },
        MarketplaceItem {
            id: "schneider-pm5300".to_string(),
            name: "Schneider PM5300".to_string(),
            description: "Digital Twin of Schneider Power Meter (Currents, Voltages). Modbus TCP.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/schneider-pm5300.yaml".to_string(),
        },
        MarketplaceItem {
            id: "zigbee-env-sensor".to_string(),
            name: "Zigbee Env Sensor".to_string(),
            description: "Digital Twin of Zigbee2MQTT Sensor (Temp, Hum, Battery). MQTT.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/zigbee-env-sensor.yaml".to_string(),
        },

        // --- Requested Industrial IoT Templates ---

        MarketplaceItem {
            id: "iot/sensors/temperature-industrial".to_string(),
            name: "Industrial Temperature Sensor".to_string(),
            description: "Industrial temperature monitoring with configurable thresholds and alerts.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/sensors/temperature-industrial.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/sensors/humidity-industrial".to_string(),
            name: "Industrial Humidity Sensor".to_string(),
            description: "Precision humidity measurement for environmental monitoring and HVAC.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/sensors/humidity-industrial.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/sensors/pressure-gauge".to_string(),
            name: "Pressure Gauge".to_string(),
            description: "High-precision pressure monitoring for pipelines and tanks.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/sensors/pressure-gauge.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/sensors/vibration-monitor".to_string(),
            name: "Vibration Monitor".to_string(),
            description: "Predictive maintenance sensor detecting anomalies in rotating machinery.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/sensors/vibration-monitor.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/sensors/flow-meter".to_string(),
            name: "Flow Meter".to_string(),
            description: "Liquid and gas flow measurement for process control.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/sensors/flow-meter.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/controllers/plc-siemens".to_string(),
            name: "PLC Controller".to_string(),
            description: "Programmable Logic Controller simulation with Modbus/OPC-UA interfaces.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/controllers/plc-siemens.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/smarthome/smart-bulb".to_string(),
            name: "Smart Bulb".to_string(),
            description: "Intelligent RGB lighting with dimming and scene control.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smarthome/smart-bulb.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/smarthome/smart-lock".to_string(),
            name: "Smart Lock".to_string(),
            description: "Connected lock with access logs and remote control.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smarthome/smart-lock.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/smarthome/thermostat-nest".to_string(),
            name: "Smart Thermostat Pro".to_string(),
            description: "Climate control with scheduling and energy reports.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smarthome/thermostat-nest.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/smarthome/motion-sensor".to_string(),
            name: "Motion Sensor".to_string(),
            description: "PIR-based occupancy detection for security and automation.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smarthome/motion-sensor.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/smarthome/ip-camera".to_string(),
            name: "IP Camera".to_string(),
            description: "Video streaming device with motion detection simulation.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/smarthome/ip-camera.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/automotive/gps-tracker".to_string(),
            name: "GPS Tracker".to_string(),
            description: "Real-time vehicle location tracking with history.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/automotive/gps-tracker.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/automotive/obd2-scanner".to_string(),
            name: "OBD-II Scanner".to_string(),
            description: "Vehicle diagnostics including engine codes, RPM, and fuel.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/automotive/obd2-scanner.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/automotive/fuel-level".to_string(),
            name: "Fuel Level Sensor".to_string(),
            description: "Tank monitoring for fleet management.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/automotive/fuel-level.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/energy/smart-meter-electric".to_string(),
            name: "Electric Smart Meter".to_string(),
            description: "Electric meter with real-time consumption and demand response.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/energy/smart-meter-electric.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/energy/wind-turbine".to_string(),
            name: "Wind Turbine".to_string(),
            description: "Turbine telemetry including wind speed and power curve.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/energy/wind-turbine.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/agriculture/soil-moisture".to_string(),
            name: "Soil Moisture Sensor".to_string(),
            description: "Agricultural sensor for irrigation optimization.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/agriculture/soil-moisture.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/manufacturing/conveyor-system".to_string(),
            name: "Conveyor System".to_string(),
            description: "Production line conveyor with speed control and item counting.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/manufacturing/conveyor-system.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/manufacturing/robot-arm-6dof".to_string(),
            name: "6-DOF Robot Arm".to_string(),
            description: "Six-axis robotic arm with joint positions and gripper status.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/manufacturing/robot-arm-6dof.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/manufacturing/sorting-machine".to_string(),
            name: "Sorting Machine".to_string(),
            description: "Automated sorting with sensors and actuators.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/manufacturing/sorting-machine.yaml".to_string(),
        },
        MarketplaceItem {
            id: "iot/gateway/edge-gateway".to_string(),
            name: "Edge Gateway".to_string(),
            description: "Protocol translator aggregating sensors via MQTT and HTTP.".to_string(),
            category: "IoT Twin".to_string(),
            logo_url: None,
            definition_url: "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/gateway/edge-gateway.yaml".to_string(),
        },
    ]
}

/// Downloads and installs a template to the specified directory.
/// Returns the path to the installed file.
pub async fn install_template(
    template_id: &str,
    output_dir: &Path,
    name_override: Option<String>,
) -> ApicentricResult<PathBuf> {
    let items = get_marketplace_items();
    let template = items.iter().find(|i| i.id == template_id).ok_or_else(|| {
        ApicentricError::Validation {
            message: format!("Template '{}' not found", template_id),
            field: Some("template".to_string()),
            suggestion: Some("Check the list of available templates".to_string()),
        }
    })?;

    println!(
        "{} Fetching template '{}' from: {}",
        "⬇️".blue(),
        template.name,
        template.definition_url
    );

    // Download content
    let client = reqwest::Client::new();
    let content = client
        .get(&template.definition_url)
        .send()
        .await
        .map_err(|e| {
            ApicentricError::network_error(
                format!("Failed to fetch template: {}", e),
                Some(&template.definition_url),
                None::<String>,
            )
        })?
        .text()
        .await
        .map_err(|e| {
            ApicentricError::network_error(
                format!("Failed to read template content: {}", e),
                Some(&template.definition_url),
                None::<String>,
            )
        })?;

    // Parse
    let value: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
        ApicentricError::validation_error(
            format!("Failed to parse template YAML: {}", e),
            None::<String>,
            Some("Check the template syntax"),
        )
    })?;

    let mut definition = if value.get("openapi").is_some() || value.get("swagger").is_some() {
        from_openapi(&value)
    } else {
        serde_yaml::from_value::<ServiceDefinition>(value).map_err(|e| {
            ApicentricError::validation_error(
                format!("Invalid service definition: {}", e),
                None::<String>,
                None::<String>,
            )
        })?
    };

    // Override name if provided, otherwise use template name (sanitized)
    if let Some(n) = name_override {
        definition.name = n;
    }

    let file_name = format!("{}.yaml", definition.name.to_lowercase().replace(' ', "-"));
    let file_path = output_dir.join(&file_name);

    // Save
    let yaml = serde_yaml::to_string(&definition).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to serialize service: {}", e), None::<String>)
    })?;

    if file_path.exists() {
        println!(
            "{} Service file '{}' already exists. Overwriting.",
            "⚠️".yellow(),
            file_path.display()
        );
    }

    std::fs::create_dir_all(output_dir).map_err(ApicentricError::Io)?;
    std::fs::write(&file_path, yaml).map_err(ApicentricError::Io)?;

    println!(
        "{} Service '{}' installed successfully at {}",
        "✨".green(),
        definition.name,
        file_path.display()
    );

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_items() {
        let items = get_marketplace_items();
        assert!(!items.is_empty());

        // Check for IoT items
        let iot_items: Vec<_> = items.iter().filter(|i| i.category == "IoT Twin").collect();
        assert_eq!(iot_items.len(), 8);

        for item in iot_items {
            assert!(item.definition_url.starts_with(
                "https://raw.githubusercontent.com/pmaojo/apicentric/main/examples/iot/"
            ));
            assert!(item.definition_url.ends_with(".yaml"));
        }
    }
}
