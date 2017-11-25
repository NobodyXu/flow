use description::name::Name;
use description::name::Named;
use loader::loader::Validate;

use std::fmt;

#[derive(Deserialize, Debug)]
pub struct Value {
    pub name: Name,
    pub datatype: Name,
    pub value: String // TODO for now....
}

// TODO figure out how to have this derived automatically for types needing it
impl Named for Value {
    fn name(&self) -> &str {
        &self.name[..]
    }
}

impl Validate for Value {
    fn validate(&self) -> Result<(), String> {
        self.value.validate()?;
        self.datatype.validate()?;

        // TODO validate the actual value, and that it matches type etc
        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Value:\n\tname: {}\n\tdatatype: {}\n\tvalue: {}", self.name, self.datatype, self.value)
    }
}