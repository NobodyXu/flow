use serde_json::Value as JsonValue;
use runnable::Runnable;
use implementation::Implementation;

const ONLY_INPUT: usize = 0;

pub struct Value {
    name: String,
    id: usize,
    initial_value: Option<JsonValue>,
    implementation: Box<Implementation>,
    value: JsonValue,
    output_routes: Vec<(&'static str, usize, usize)>,
}

impl Value {
    pub fn new(name: String,
               _num_inputs: usize,
               id: usize,
               implementation: Box<Implementation>,
               initial_value: Option<JsonValue>,
               output_routes: Vec<(&'static str, usize, usize)>) -> Value {
        Value {
            name,
            id,
            initial_value,
            implementation,
            value: JsonValue::Null,
            output_routes,
        }
    }
}

impl Runnable for Value {
    fn name(&self) -> &str {
        &self.name
    }

    fn number_of_inputs(&self) -> usize { 1 }

    fn id(&self) -> usize { self.id }

    /*
        If an initial value is defined then write it to the current value.
        Return true if ready to run as all inputs (single in this case) are satisfied.
    */
    fn init(&mut self) -> bool {
        let value = self.initial_value.clone();
        if let Some(v) = value {
            debug!("\tValue initialized by writing '{:?}' to input", &v);
            self.write_input(ONLY_INPUT, v);
        }
        self.inputs_satisfied()
    }

    /*
        Update the value stored - this should only be called when the value has already been
        consumed by all the listeners and hence it can be overwritten.
    */
    fn write_input(&mut self, _input_number: usize, input_value: JsonValue) {
        self.value = input_value;
    }

    /*
        Responds true if all inputs have been satisfied - false otherwise
    */
    fn inputs_satisfied(&self) -> bool {
        !self.value.is_null()
    }

    /*
        Consume the inputs and pass them to the actual implementation
    */
    fn run(&mut self) -> JsonValue {
        let input = self.value.take();
        self.implementation.run(vec!(input))
    }

    fn output_destinations(&self) -> &Vec<(&'static str, usize, usize)> {
        &self.output_routes
    }
}

#[cfg(test)]
mod test {
    use super::Value;
    use super::super::implementation::Implementation;
    use serde_json::Value as JsonValue;

    struct TestValue;

    impl Implementation for TestValue {
        fn run(&self, mut inputs: Vec<JsonValue>) -> JsonValue {
            inputs.remove(0)
        }
    }

    #[test]
    fn destructure_output_base_route() {
        let json = json!("my_value");
        let value = Value {
            name: "test_value".to_string(),
            id: 0,
            initial_value: Some(json.clone()),
            implementation: Box::new(TestValue),
            value: json.clone(),
            output_routes: vec!(("", 1, 0)),
        };

        let output = value.implementation.run(vec!(json.clone()));

        assert_eq!(output.pointer("").unwrap(), "my_value");
    }

    #[test]
    fn destructure_json_value() {
        let json: JsonValue = json!({ "sub_route": "sub_value" });

        let value = Value {
            name: "test_value".to_string(),
            id: 0,
            initial_value: Some(json.clone()),
            implementation: Box::new(TestValue),
            value: json.clone(),
            output_routes: vec!(("", 1, 0), ("sub_route", 2, 0)),
        };

        let output = value.implementation.run(vec!(json));
        assert_eq!(output.pointer("/sub_route").unwrap(), "sub_value");
    }
}