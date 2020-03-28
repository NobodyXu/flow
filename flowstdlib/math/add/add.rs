use flow_impl::{Implementation, RUN_AGAIN, RunAgain};
use flow_impl_derive::FlowImpl;
use serde_json::Value;
use serde_json::Value::Number;
use serde_json::Value::String;

#[derive(FlowImpl)]
/// Add two inputs to produce a new output
///
/// ## Include using
/// ```toml
/// [[process]]
/// alias = "add"
/// source = "lib://flowstdlib/math/add"
/// ```
///
/// ## Inputs
/// * `i1` - first input of type `Number`
/// * `i2` - second input of type `Number`
///
/// ## Outputs
/// * `sum` of `i1` and `i2` of type `Number`
#[derive(Debug)]
pub struct Add;

impl Implementation for Add {
    fn run(&self, inputs: &Vec<Vec<Value>>) -> (Option<Value>, RunAgain) {
        let mut sum = None;

        let input_a = &inputs[0];
        let input_b = &inputs[1];

        let mut output_map = serde_json::Map::new();

        match (&input_a[0], &input_b[0]) {
            (&Number(ref a), &Number(ref b)) => {
                sum = if a.is_i64() && b.is_i64() {
                    match a.as_i64().unwrap().checked_add(b.as_i64().unwrap()) {
                        Some(result) => Some(Value::Number(serde_json::Number::from(result))),
                        None => None
                    }
                } else if a.is_u64() && b.is_u64() {
                    match a.as_u64().unwrap().checked_add(b.as_u64().unwrap()) {
                        Some(result) => Some(Value::Number(serde_json::Number::from(result))),
                        None => None
                    }
                } else if a.is_f64() || b.is_f64() {
                    Some(Value::Number(serde_json::Number::from_f64(a.as_f64().unwrap() + b.as_f64().unwrap()).unwrap()))
                } else {
                    println!("Unsupported input value types");
                    None
                };
            }
            (&String(ref a), &String(ref b)) => {
                let i1 = a.parse::<i64>().unwrap();
                let i2 = b.parse::<i64>().unwrap();
                let o1 = i1 + i2;
                sum = Some(Value::String(o1.to_string()));
            }
            (_, _) => println!("Unsupported input value types")
        }

        if sum.is_some() {
            output_map.insert("sum".into(), sum.unwrap());
            output_map.insert("i1".into(), input_a[0].clone());
            output_map.insert("i2".into(), input_b[0].clone());

            let output = Value::Object(output_map);

            (Some(output), RUN_AGAIN)
        } else {
            (None, RUN_AGAIN)
        }
    }
}

#[cfg(test)]
mod test {
    use flow_impl::Implementation;
    use serde_json::Value;
    use serde_json::Value::Number;

    use super::Add;

    fn get_inputs(pair: &(Value, Value, Option<Value>)) -> Vec<Vec<Value>> {
        vec!(vec!(pair.0.clone()), vec!(pair.1.clone()))
    }

    // 0 plus 0
    // 0 plus negative
    // negative plus 0
    // 0 plus positive
    // positive plus zero
    // positive plus positive
    // negative plus negative
    // positive plus negative
    // negative plus positive
    // overflow positive
    // overflow negative

    // all of those for:
    // integer + integer
    // float plus float
    // float plus integer
    // integer plus float

    // all of those as numbers and strings
    #[test]
    fn test_adder() {
        let integer_test_set = vec!(
            (Number(serde_json::Number::from(0)), Number(serde_json::Number::from(0)), Some(Number(serde_json::Number::from(0)))),
            (Number(serde_json::Number::from(0)), Number(serde_json::Number::from(-10)), Some(Number(serde_json::Number::from(-10)))),
            (Number(serde_json::Number::from(-10)), Number(serde_json::Number::from(0)), Some(Number(serde_json::Number::from(-10)))),
            (Number(serde_json::Number::from(0)), Number(serde_json::Number::from(10)), Some(Number(serde_json::Number::from(10)))),
            (Number(serde_json::Number::from(10)), Number(serde_json::Number::from(0)), Some(Number(serde_json::Number::from(10)))),
            (Number(serde_json::Number::from(10)), Number(serde_json::Number::from(20)), Some(Number(serde_json::Number::from(30)))),
            (Number(serde_json::Number::from(-10)), Number(serde_json::Number::from(-20)), Some(Number(serde_json::Number::from(-30)))),
            (Number(serde_json::Number::from(10)), Number(serde_json::Number::from(-20)), Some(Number(serde_json::Number::from(-10)))),
            (Number(serde_json::Number::from(-10)), Number(serde_json::Number::from(20)), Some(Number(serde_json::Number::from(10)))),
            (Number(serde_json::Number::from(4660046610375530309 as i64)), Number(serde_json::Number::from(7540113804746346429 as i64)), None),
            (Number(serde_json::Number::from(-4660046610375530309 as i64)), Number(serde_json::Number::from(-7540113804746346429 as i64)), None),
        );

        let added = Add {};

        for ref test in integer_test_set {
            println!("Testing add of {:?}", test);
            let (output, again) = added.run(&get_inputs(test));

            assert_eq!(true, again);

            match output {
                Some(outputs) => {
                    assert_eq!(outputs.pointer("/i1").unwrap(), &test.0);
                    assert_eq!(outputs.pointer("/i2").unwrap(), &test.1);
                    assert_eq!(outputs.pointer("/sum"), test.2.as_ref());
                }
                None => {
                    assert!(test.2.is_none())
                }
            }
        }
    }
}