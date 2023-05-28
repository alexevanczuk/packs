pub(crate) fn to_sentence(list: &Vec<String>) -> String {
    let mut file_string = String::new();
    for (i, file) in list.iter().enumerate() {
        let second_to_last = i == list.len() - 1;
        if i == 0 {
            file_string.push_str(file);
        } else if second_to_last && list.len() > 2 {
            file_string.push_str(", and ");
            file_string.push_str(file);
        } else if second_to_last {
            file_string.push_str(" and ");
            file_string.push_str(file);
        } else {
            file_string.push_str(", ");
            file_string.push_str(file);
        }
    }

    file_string
}
