use model::flow::Flow;
use loader::loader::Loader;
use model::function::Function;
use url::Url;

pub struct FlowYamlLoader;

impl Loader for FlowYamlLoader {
    // TODO define our own errors types? so we can return errors from lower down directly
    fn load_flow(&self, _contents: &str) -> Result<Flow, String> {
//        let docs = YamlLoader::load_from_str(&contents).unwrap();
//        let doc = &docs[0];

        let flow =
            Flow {
                source_url: Url::parse("fake").unwrap(),
                route: "fake/fake".to_string(),
                name: "fake".to_string(),
                flow_refs: None,
                connections: None,
                inputs: None,
                outputs: None,
                function_refs: None,
                values: None,
            };

        Ok(flow)
    }

    fn load_function(&self, _contents: &str) -> Result<Function, String> {
//        let docs = YamlLoader::load_from_str(&contents).unwrap();
//        let doc = &docs[0];

        let function = Function {
            name: "fake".to_string(),
            route: "fake/fake".to_string(),
            inputs: None,
            outputs: None,
        };

        Ok(function)
    }
}
