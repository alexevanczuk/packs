pub(crate) mod parser;

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::packs::parsing::erb::packwerk::parser::process_from_contents;
    use crate::packs::parsing::Range;
    use crate::packs::UnresolvedReference;

    #[test]
    fn trivial_case() {
        let contents: String = String::from("<%= Foo %>");
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range::default()
            }],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }

    #[test]
    fn multiple_references() {
        let contents: String = String::from("<%= Foo %><%= Bar %>");
        assert_eq!(
            vec![
                UnresolvedReference {
                    name: String::from("Foo"),
                    namespace_path: vec![],
                    location: Range::default()
                },
                UnresolvedReference {
                    name: String::from("Bar"),
                    namespace_path: vec![],
                    location: Range::default()
                }
            ],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }
    #[test]
    fn multiline_erb() {
        let contents: String = String::from(
            "/
<%
    Foo
%>
        ",
        );
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range::default()
            }],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }

    #[test]
    fn erb_with_leading_hyphen_syntax() {
        let contents: String = String::from(
            "/
  <%- Foo %>
    <%= do_thing() %>
  <%- end %>
        ",
        );
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range::default()
            }],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }

    #[test]
    fn erb_with_trailing_hyphen_syntax() {
        let contents: String = String::from(
            "/
<% Foo %>
<div>
  <div>
    <p>
      <% if condition %>
      <% else %>
      <% end -%>
    </p>
  </div>
</div>
        ",
        );
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range::default()
            }],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }

    #[test]
    fn complex_multiline_erb() {
        let contents: String = String::from(
            "/
<%
    # Comment
    # Comment
    Foo
    # Comment
    # Comment
%>
        ",
        );
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range::default()
            }],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }

    #[test]
    fn complex_erb() {
        let contents: String = String::from(
            "/
<!DOCTYPE html>
<html>
<head>
  <title>ERB Snippet</title>
</head>
<body>
  <% if Foo %>
    <h1>Hello, World!</h1>
  <% else %>
    <p>Welcome to the ERB snippet!</p>
  <% end %>

  <% unless Bar.empty? %>
    <ul>
      <% Baz.each do |item| %>
        <li><%= item %></li>
      <% end %>
    </ul>
  <% end %>

  <% for i in Boo %>
    <p>Iteration <%= Bee %></p>
  <% end %>
</body>
</html>
        ",
        );
        assert_eq!(
            vec![
                UnresolvedReference {
                    name: String::from("Foo"),
                    namespace_path: vec![],
                    location: Range::default()
                },
                UnresolvedReference {
                    name: String::from("Bar"),
                    namespace_path: vec![],
                    location: Range::default()
                },
                UnresolvedReference {
                    name: String::from("Baz"),
                    namespace_path: vec![],
                    location: Range::default()
                },
                UnresolvedReference {
                    name: String::from("Boo"),
                    namespace_path: vec![],
                    location: Range::default()
                },
                UnresolvedReference {
                    name: String::from("Bee"),
                    namespace_path: vec![],
                    location: Range::default()
                }
            ],
            process_from_contents(contents, &PathBuf::from("path/to/file.rb"))
                .unresolved_references
        );
    }
}
