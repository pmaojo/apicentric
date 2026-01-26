# Catálogo de Plantillas Apicentric

Apicentric ofrece una amplia gama de plantillas listas para usar, desde APIs SaaS populares hasta Gemelos Digitales IoT complejos. Utiliza el comando `apicentric simulator start --template <ID>` para desplegar cualquiera de estas plantillas en segundos.

## ☁️ APIs SaaS
Simulaciones de servicios populares para desarrollo y pruebas.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Stripe API** | API simulada de Stripe para pagos, clientes y suscripciones. | `apicentric simulator start --template stripe` |
| **Slack API** | API Web de Slack simulada para mensajería y canales. | `apicentric simulator start --template slack` |
| **GitHub API** | API REST de GitHub simulada para repositorios e incidencias. | `apicentric simulator start --template github` |
| **OpenAI API** | API de OpenAI simulada para chat completions y embeddings. | `apicentric simulator start --template openai` |
| **Kubernetes API** | API de Kubernetes (Core) para pods, servicios y despliegues. | `apicentric simulator start --template kubernetes` |
| **SendGrid API** | API v3 de SendGrid simulada para envío de emails. | `apicentric simulator start --template sendgrid` |
| **DigitalOcean API** | API pública de DigitalOcean para droplets y volúmenes. | `apicentric simulator start --template digitalocean` |

## 🏭 Industrial IoT
Sensores y controladores para entornos industriales.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Sensor de Temperatura Industrial** | Monitoreo de temperatura con umbrales y alertas configurables. | `apicentric simulator start --template iot/sensors/temperature-industrial` |
| **Sensor de Humedad Industrial** | Medición de humedad de precisión para HVAC y monitoreo ambiental. | `apicentric simulator start --template iot/sensors/humidity-industrial` |
| **Manómetro** | Monitoreo de presión de alta precisión para tuberías y tanques. | `apicentric simulator start --template iot/sensors/pressure-gauge` |
| **Monitor de Vibración** | Sensor de mantenimiento predictivo para maquinaria rotativa. | `apicentric simulator start --template iot/sensors/vibration-monitor` |
| **Caudalímetro** | Medición de flujo de líquidos y gases para control de procesos. | `apicentric simulator start --template iot/sensors/flow-meter` |
| **Controlador PLC** | Simulación de PLC Siemens con interfaces Modbus/OPC-UA. | `apicentric simulator start --template iot/controllers/plc-siemens` |
| **Bomba Industrial** | Gemelo Digital de una bomba industrial (RPM, flujo, temp). Modbus TCP. | `apicentric simulator start --template industrial-pump` |

## 🏠 Hogar Inteligente (Smart Home)
Dispositivos conectados para domótica.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Bombilla Inteligente** | Iluminación RGB inteligente con atenuación y control de escenas. | `apicentric simulator start --template iot/smarthome/smart-bulb` |
| **Cerradura Inteligente** | Cerradura conectada con registros de acceso y control remoto. | `apicentric simulator start --template iot/smarthome/smart-lock` |
| **Termostato Nest** | Control climático inteligente con programación y reportes de energía. | `apicentric simulator start --template iot/smarthome/thermostat-nest` |
| **Sensor de Movimiento** | Detección de ocupación PIR para seguridad y automatización. | `apicentric simulator start --template iot/smarthome/motion-sensor` |
| **Cámara IP** | Dispositivo de streaming de video con simulación de detección de movimiento. | `apicentric simulator start --template iot/smarthome/ip-camera` |
| **Termostato Básico** | Termostato inteligente básico (temperatura, humedad). MQTT. | `apicentric simulator start --template smart-thermostat` |
| **Philips Hue** | API simulada de Philips Hue Bridge para luces inteligentes. | `apicentric simulator start --template philips-hue` |
| **Sonos** | API simulada de control Sonos para altavoces inteligentes. | `apicentric simulator start --template sonos` |

## 🚗 Automotriz
Telemetría y diagnóstico vehicular.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Rastreador GPS** | Seguimiento de ubicación en tiempo real con historial. | `apicentric simulator start --template iot/automotive/gps-tracker` |
| **Escáner OBD-II** | Diagnósticos vehiculares (códigos de motor, RPM, combustible). | `apicentric simulator start --template iot/automotive/obd2-scanner` |
| **Sensor de Nivel de Combustible** | Monitoreo de tanques para gestión de flotas. | `apicentric simulator start --template iot/automotive/fuel-level` |

## ⚡ Energía
Gestión y monitoreo de recursos energéticos.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Medidor Inteligente Eléctrico** | Medidor con consumo en tiempo real y respuesta a la demanda. | `apicentric simulator start --template iot/energy/smart-meter-electric` |
| **Inversor Solar** | Simulación de inversor FV con potencia y eficiencia. | `apicentric simulator start --template iot/energy/solar-inverter` |
| **Turbina Eólica** | Telemetría de turbina (viento, curva de potencia). | `apicentric simulator start --template iot/energy/wind-turbine` |
| **Sistema Victron Energy** | Gemelo Digital de dispositivo Victron GX (Voltaje Batería, SoC, PV). Modbus TCP. | `apicentric simulator start --template victron-energy-system` |
| **Schneider PM5300** | Medidor de potencia Schneider (Corrientes, Voltajes). Modbus TCP. | `apicentric simulator start --template schneider-pm5300` |
| **Medidor Inteligente (Genérico)** | Gemelo digital de medidor de energía básico. MQTT. | `apicentric simulator start --template smart-meter` |

## 🌾 Agricultura
Tecnología para el campo.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Sensor de Humedad del Suelo** | Sensor para optimización de riego. | `apicentric simulator start --template iot/agriculture/soil-moisture` |
| **Estación Meteorológica** | Estación multiparámetro (viento, temp, humedad). MQTT. | `apicentric simulator start --template iot/agriculture/weather-station` |

## 🏭 Manufactura
Automatización de líneas de producción.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Sistema de Cinta Transportadora** | Transportador con control de velocidad y conteo. | `apicentric simulator start --template iot/manufacturing/conveyor-system` |
| **Brazo Robot 6-DOF** | Brazo robótico de seis ejes con estado del gripper. | `apicentric simulator start --template iot/manufacturing/robot-arm-6dof` |
| **Máquina Clasificadora** | Clasificación automatizada con sensores y actuadores. | `apicentric simulator start --template iot/manufacturing/sorting-machine` |

## 🌐 Conectividad y Otros
Infraestructura de red y dispositivos varios.

| Nombre | Descripción | Comando |
|--------|-------------|---------|
| **Edge Gateway** | Traductor de protocolos (MQTT, HTTP, CoAP). | `apicentric simulator start --template iot/gateway/edge-gateway` |
| **Lector RFID Zebra FX9600** | Gemelo Digital de lector RFID con conector IoT. | `apicentric simulator start --template iot/rfid/zebra-fx9600` |
| **Sensor Zigbee** | Sensor ambiental Zigbee2MQTT (Temp, Hum, Batería). | `apicentric simulator start --template zigbee-env-sensor` |
| **Acme Smart Sensor** | Sensor IoT genérico para lecturas de estado. | `apicentric simulator start --template acme-sensor` |
| **PetStore** | Ejemplo estándar de API Swagger PetStore. | `apicentric simulator start --template petstore` |
