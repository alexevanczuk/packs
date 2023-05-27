pub(crate) fn to_sentence(list: Vec<String>) -> String {
    let mut file_string = String::new();
    // Extract this to src/string_helpers.rs to_sentence function and import it into this
    // and invoke it.
    for (i, file) in list.iter().enumerate() {
        if i == 0 {
            file_string.push_str(file);
        } else if i == list.len() - 1 {
            file_string.push_str(", and ");
            file_string.push_str(file);
        } else {
            file_string.push_str(", ");
            file_string.push_str(file);
        }
    }

    file_string
}
