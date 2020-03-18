use crate::compiler::loader::Deserializer;
use crate::errors::*;
use crate::model::process::Process;

pub struct FlowYamlLoader;

// NOTE: Indexes are one-based
impl Deserializer for FlowYamlLoader {
    fn deserialize(&self, contents: &str, url: Option<&str>) -> Result<Process> {
        serde_yaml::from_str(contents)
            .chain_err(|| format!("Error deserializing Yaml from: '{}'",
                                  url.or_else(|| { Some("URL unknown") } ).unwrap()))

    }

    fn name(&self) -> &'static str { "Yaml" }
}


#[cfg(test)]
mod test {
    use crate::compiler::loader::Deserializer;

    use super::FlowYamlLoader;

    #[test]
    fn invalid_yaml() {
        let deserializer = FlowYamlLoader {};

        match deserializer.deserialize("{}", None) {
            Ok(_) => assert!(false, "Should not have parsed correctly as is invalid JSON"),
            Err(_) => assert!(true, "Should produce syntax error at (1,1)")
        };
    }

    #[test]
    fn simple_context_loads() {
        let flow_description = "
flow: hello-world-simple-toml

version: 0.0.0
author_name: unknown
author_email: unknown@unknown.com
";

        let yaml = FlowYamlLoader {};
        let flow = yaml.deserialize(&flow_description.replace("'", "\""), None);
        println!("{:?}", flow);
        assert!(flow.is_ok());
    }
}