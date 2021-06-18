pub struct Expandable {
    //If nonzero, this can be expanded
    pub variable_reference: u64,
    //This contains all known child variables
    pub variables: Vec<Expandable>,
    //This contains the line in the Variables buffer this variable is displayed
    pub line_no: u64,
    //This determines if this Expandable is a scope or a variable
    pub is_var: bool,
    //This is the contents of the Expandable
    pub contents: json::JsonValue,
}
