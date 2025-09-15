use crate::domain::contract_testing::ValidationScenario;
use crate::domain::ports::contract::{ServiceSpec, ServiceSpecLoader, SpecLoaderError};

/// Extracts validation scenarios from a service specification using a `ServiceSpecLoader` port.
pub struct ScenarioExtractor<L: ServiceSpecLoader> {
    loader: L,
}

impl<L: ServiceSpecLoader> ScenarioExtractor<L> {
    /// Create a new extractor with the given service spec loader.
    pub fn new(loader: L) -> Self {
        Self { loader }
    }

    /// Load a service specification and extract its validation scenarios.
    pub async fn extract(&self, spec_path: &str) -> Result<Vec<ValidationScenario>, SpecLoaderError> {
        let spec: ServiceSpec = self.loader.load(spec_path).await?;
        self.loader.extract_scenarios(&spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;

    struct FakeLoader;

    #[async_trait]
    impl ServiceSpecLoader for FakeLoader {
        async fn load(&self, _path: &str) -> Result<ServiceSpec, SpecLoaderError> {
            Ok(ServiceSpec {
                name: "svc".to_string(),
                port: 80,
                base_path: "/".to_string(),
                fixtures: serde_json::json!({}),
                endpoints: vec![crate::domain::ports::contract::EndpointSpec {
                    path: "/ping".to_string(),
                    method: crate::domain::contract_testing::HttpMethod::GET,
                    conditions: vec![],
                    response: crate::domain::ports::contract::ResponseSpec {
                        status: 200,
                        headers: HashMap::new(),
                        body_template: String::new(),
                    },
                }],
            })
        }

        async fn validate(&self, _spec: &ServiceSpec) -> Result<(), SpecLoaderError> {
            Ok(())
        }

        fn extract_scenarios(
            &self,
            spec: &ServiceSpec,
        ) -> Result<Vec<ValidationScenario>, SpecLoaderError> {
            Ok(spec
                .endpoints
                .iter()
                .enumerate()
                .map(|(i, ep)| {
                    ValidationScenario::new(
                        format!("s{}", i),
                        ep.path.clone(),
                        ep.method.clone(),
                    )
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn extracts_scenarios_from_spec() {
        let extractor = ScenarioExtractor::new(FakeLoader);
        let scenarios = extractor.extract("spec.yaml").await.unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].path(), "/ping");
    }
}

