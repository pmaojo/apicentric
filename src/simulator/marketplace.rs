use serde::{Deserialize, Serialize};

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
    ]
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
