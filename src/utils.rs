use convert_case::{Case, Casing};

pub fn create_interface_name(name: &str, from_db: bool) -> String {
    let pascal_case_name = name.to_case(Case::Pascal);

    if from_db {
        format!("Out{pascal_case_name}")
    } else {
        format!("In{pascal_case_name}")
    }
}