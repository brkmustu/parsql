pub(crate) fn sanitize_identifier(input: &str) -> String {
    input.replace(|c: char| !c.is_alphanumeric() && c != '_', "")
}

pub(crate) fn sanitize_value(input: &str) -> String {
    input.replace('\'', "''")
} 