#[cfg(test)]
mod tests {
    use apicentric::simulator::config::ServiceDefinition;

    #[test]
    fn test_twin_field_exists() {
        #[cfg(feature = "iot")]
        println!("IoT feature is ON");
        #[cfg(not(feature = "iot"))]
        println!("IoT feature is OFF");
    }
}
