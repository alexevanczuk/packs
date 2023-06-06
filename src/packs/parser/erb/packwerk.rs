pub(crate) mod extractor;

#[cfg(test)]
mod tests {
    use crate::packs::parser::erb::packwerk::extractor::extract_from_contents;
    use crate::packs::Range;
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
            extract_from_contents(contents)
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
            extract_from_contents(contents)
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
            extract_from_contents(contents)
        );
    }

    #[test]
    fn erb_with_hyphen_syntax() {
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
            extract_from_contents(contents)
        );
    }

    #[test]
    fn html_in_erb() {
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
            extract_from_contents(contents)
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
            extract_from_contents(contents)
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
            extract_from_contents(contents)
        );
    }
}
