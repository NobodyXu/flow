use description::name::Name;
use description::connection::Connection;
use description::connection::IO;
use description::function::FunctionRef;
use description::function::Function;
use description::value::Value;
use loader::loader::Validate;

use std::fmt;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct FlowRef {
    pub name: Name,
    pub source: String
}

impl fmt::Display for FlowRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FlowRef:\n\tname: {}\n\tsource: {}", self.name, self.source)
    }
}

#[derive(Deserialize)]
pub struct Flow {
    #[serde(skip_deserializing)]
    pub source: PathBuf,
    pub name: Name,
    pub flow: Option<Vec<FlowRef>>,
    pub io: Option<Vec<IO>>,
    pub value: Option<Vec<Value>>,
    pub function: Option<Vec<FunctionRef>>,
    pub connection: Option<Vec<Connection>>,
    #[serde(skip_deserializing)]
    pub flows: Vec<Flow>,
    #[serde(skip_deserializing)]
    pub functions: Vec<Function>
}

impl Validate for Flow {
    // check the correctness of all the fields in this flow, prior to loading sub-elements
    fn validate(&self) -> Result<(), String> {
        self.name.validate()?;

        // TODO all this!

        // Definitions at this level
        // check the IOs defined in this flow are of a valid format
        // Check values defined in this flow are of a valid format

        // References used
        // Check flow references found are of a valid format....
        // Check function references are of a valid format

        // Check all connections are of a valid format

        // Internal consistency
        // Check connections referring to IOs of this flow match those IOs
        // check connections referring to values of this flow match those values

        Ok(())
    }
}

impl Flow {
    // now that all is loaded, check all is OK
    pub fn verify(&self) -> Result<(), String> {
        // TODO Check the connections and connect them up with refs?
        // pub connection: Option<Vec<Connection>>,

        Ok(())
    }
}

impl fmt::Display for Flow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nFlow:\n\tname: {}\n\tFlowRefs: {:?}\n\tvalue: {:?}\n\tio: {:?}\n\tFunctionRefs: {:?}\n\tconnection: {:?}",
               self.name, self.flow, self.value, self.io, self.function, self.connection)
    }
}