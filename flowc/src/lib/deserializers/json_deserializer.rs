use serde::Deserialize;
use url::Url;

use crate::compiler::loader::Deserializer;
use crate::errors::*;

pub struct JsonDeserializer;

impl<'a, P> Deserializer<'a, P> for JsonDeserializer
where
    P: Deserialize<'a>,
{
    fn deserialize(&self, contents: &'a str, url: Option<&Url>) -> Result<P> {
        serde_json::from_str(contents).chain_err(|| {
            format!(
                "Error deserializing Json from: '{}'",
                url.map_or("URL unknown".to_owned(), |u| u.to_string())
            )
        })
    }

    fn name(&self) -> &str {
        "Json"
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::loader::Deserializer;
    use crate::model::process::Process;

    use super::JsonDeserializer;

    #[test]
    fn invalid_json() {
        let json = &JsonDeserializer {} as &dyn Deserializer<Process>;

        if json.deserialize("=", None).is_ok() {
            panic!("Should not have parsed correctly as is invalid JSON");
        };
    }

    #[test]
    fn simplest_context_loads() {
        let flow_description = "{
    'flow': 'hello-world-simple-toml',
    'process': [
        {
            'alias': 'print',
            'source': 'lib://flowruntime/stdio/stdout.toml',
            'input': {
                'default': {
                    'once': 'hello'
                }
            }
        }
    ]
}";
        let toml = &JsonDeserializer {} as &dyn Deserializer<Process>;
        let flow = toml.deserialize(&flow_description.replace("'", "\""), None);
        assert!(flow.is_ok());
    }

    #[test]
    fn simple_context_loads() {
        let flow_description = "{
    'flow': 'hello-world-simple-toml',
    'process': [
        {
            'alias': 'message',
            'source': 'lib://flowstdlib/data/buffer.toml',
            'input': {
                'default': {
                    'once': 'hello'
                }
            }
        },
        {
            'alias': 'print',
            'source': 'lib://flowruntime/stdio/stdout.toml'
        }
    ],
    'connection': [
        {
            'from': 'message',
            'to': 'print'
        }
    ]
}";
        let json = &JsonDeserializer {} as &dyn Deserializer<Process>;
        let flow = json.deserialize(&flow_description.replace("'", "\""), None);
        assert!(flow.is_ok());
    }

    #[test]
    fn invalid_context_fails() {
        let flow_description = "{
    'flow': 'hello-world-simple-toml',
    'process': [
        {
            'alias': 'message',
            'source': 'lib://flowstdlib/data/buffer.toml',
            'input': {
                'default': {
                    'once': 'hello'
                }
            }
        },
        {
            'alias': 'print',
            'source': 'lib://flowruntime/stdio/stdout.toml'
        }
    ],
    'connection': [
        {\
            'from': 'message'
        }
    ]
}";
        let json = &JsonDeserializer {} as &dyn Deserializer<Process>;
        let flow = json.deserialize(&flow_description.replace("'", "\""), None);
        assert!(flow.is_err());
    }
}
