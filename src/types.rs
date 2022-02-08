pub struct Scope {
    // If nonzero, this can be expanded
    pub variable_reference: u64,
    // This is the contents of the Scope
    pub contents: json::JsonValue,
}

#[derive(Debug)]
pub struct Variable {
    // If nonzero, this can be expanded
    pub variable_reference: u64,
    // This contains the variable reference of this variable's "parent"
    pub par_variable_reference: u64,
    // This is the contents of the Variable
    pub contents: json::JsonValue,
}
