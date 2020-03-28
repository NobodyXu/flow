use flow_impl::{Implementation, RUN_AGAIN, RunAgain};
use flow_impl_derive::FlowImpl;
use serde_json::Value;

#[derive(FlowImpl)]
/// Accumulate input values into an array upto the limit specified
///
/// ## Include using
/// ```toml
/// [[process]]
/// alias = "accumulate"
/// source = "lib://flowstdlib/data/accumulate"
/// ```
///
/// ## Input
/// name = "values"
/// * The stream of input values to accumulate into an array
///
/// ## Input
/// name = "partial"
/// type = "Array"
/// * A partial array, to use in the accumulation, adding next input `value` to it
///
/// ## Input
/// name = "chunk_size"
/// type = "Number"
/// * The size of the Array we want to create
///
/// ## Outputs
/// name = "chunk"
/// type = "Array"
/// * The accumulated Array of inputs of size `limit` or more
///
/// ## Outputs
/// name = "partial"
/// type = "Array"
/// * The partially accumulated array, of size smaller than `chunk_size`
///
/// ## Outputs
/// name = "limit"
/// type = "Number"
/// * The limit, output for use downstream or in loop-back
#[derive(Debug)]
pub struct Accumulate;

impl Implementation for Accumulate {
    fn run(&self, inputs: &Vec<Vec<Value>>) -> (Option<Value>, RunAgain) {
        let mut values = inputs[0].clone();
        let mut input1 = inputs[1][0].clone();
        let accumulated = input1.as_array_mut().unwrap();
        let limit = inputs[2][0].clone();
        accumulated.append(&mut values);

        let mut output_map = serde_json::Map::new();

        if accumulated.len() >= limit.as_u64().unwrap() as usize {
            // TODO could pass on any extra elements beyond chunk size in 'partial'
            // and also force chunk size to be exact....
            output_map.insert("chunk".into(), Value::Array(accumulated.clone()));
            output_map.insert("partial".into(), Value::Array(vec!()));
        } else {
            output_map.insert("partial".into(), Value::Array(accumulated.clone()));
        }

        output_map.insert("chunk_size".into(), limit.clone());

        let output = Value::Object(output_map);

        (Some(output), RUN_AGAIN)
    }
}

#[cfg(test)]
mod test {
    use flow_impl::Implementation;
    use serde_json::{Number, Value};

    #[ignore]
    #[test]
    fn remove_1() {
        let array: Vec<Value> = vec!(Value::Array(vec!(Value::Number(Number::from(1)),
                                                       Value::Number(Number::from(2)))));
        let value = vec!(Value::Number(Number::from(1)));

        let accumulator = super::Accumulate {};
        let (result, _) = accumulator.run(&vec!(value, array));

        assert_eq!(result.unwrap(), Value::Array(vec!(Value::Number(Number::from(2)))));
    }
}