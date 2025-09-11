use crate::packs::file_utils::file_read_contents;
use crate::packs::parsing::ruby::parse_utils::extract_sigils_from_contents;
use crate::packs::{
    parsing::{
        ruby::parse_utils::{
            fetch_const_const_name, fetch_const_name, fetch_node_location,
            get_constant_assignment_definition, get_definition_from,
            get_reference_from_active_record_association, loc_to_range,
        },
        ParsedDefinition, UnresolvedReference,
    },
    Configuration, ProcessedFile,
};
use lib_ruby_parser::{
    nodes, traverse::visitor::Visitor, Node, Parser, ParserOptions,
};
use line_col::LineColLookup;
use std::path::Path;

struct ReferenceCollector<'a> {
    pub references: Vec<UnresolvedReference>,
    pub definitions: Vec<ParsedDefinition>,
    pub current_namespaces: Vec<String>,
    pub line_col_lookup: LineColLookup<'a>,
    pub behavioral_change_in_namespace: bool,
    pub custom_associations: Vec<String>,
    pub is_spec_file: bool,
}

impl<'a> Visitor for ReferenceCollector<'a> {
    fn on_class(&mut self, node: &nodes::Class) {
        // We're not collecting definitions, so no need to visit the class definitioname);
        let namespace_result = fetch_const_name(&node.name);
        // For now, we simply exit and stop traversing if we encounter an error when fetching the constant name of a class
        // We can iterate on this if this is different than the packwerk implementation
        if namespace_result.is_err() {
            return;
        }

        let namespace = namespace_result.unwrap();

        if let Some(inner) = node.superclass.as_ref() {
            self.visit(inner);
        }
        let definition_loc = fetch_node_location(&node.name).unwrap();
        let location = loc_to_range(definition_loc, &self.line_col_lookup);

        let definition = get_definition_from(
            &namespace,
            &self.current_namespaces,
            &location,
        );

        // Note – is there a way to use lifetime specifiers to get rid of this and
        // just keep current namespaces as a vector of string references or something else
        // more efficient?
        self.current_namespaces.push(namespace);

        // Each time we open up a new class/module, we reset the behavioral change flag
        let previous_behavioral_change = self.behavioral_change_in_namespace;
        self.behavioral_change_in_namespace = false;

        if let Some(inner) = &node.body {
            self.visit(inner);
        }

        if self.behavioral_change_in_namespace {
            self.definitions.push(definition);
        }

        // When we're done visiting the class/module, we restore the previous behavioral change flag
        // to account for nested class/module definitions
        self.behavioral_change_in_namespace = previous_behavioral_change;

        self.current_namespaces.pop();
    }

    fn on_send(&mut self, node: &nodes::Send) {
        if node.method_name == "private_constant" || self.is_spec_file {
            // `private_constant`, RSpec methods, and anything inside RSpec describe blocks
            // are not considered to be behavioral changes
            lib_ruby_parser::traverse::visitor::visit_send(self, node);
        } else {
            self.behavioral_change_in_namespace = true;

            let association_reference =
                get_reference_from_active_record_association(
                    node,
                    &self.current_namespaces,
                    &self.line_col_lookup,
                    &self.custom_associations,
                );

            if let Some(association_reference) = association_reference {
                self.references.push(association_reference);
            }

            lib_ruby_parser::traverse::visitor::visit_send(self, node);
        }
    }

    fn on_casgn(&mut self, node: &nodes::Casgn) {
        let definition = get_constant_assignment_definition(
            node,
            self.current_namespaces.to_owned(),
            &self.line_col_lookup,
        );

        if let Some(definition) = definition {
            self.definitions.push(definition);
        }

        if let Some(v) = node.value.to_owned() {
            self.visit(&v);
        } else {
            // We don't handle constant assignments as part of a multi-assignment yet,
            // e.g. A, B = 1, 2
            // See the documentation for nodes::Casgn#value for more info.
        }
    }

    fn on_module(&mut self, node: &nodes::Module) {
        let namespace = fetch_const_name(&node.name).unwrap_or("".to_owned());
        let definition_loc = fetch_node_location(&node.name).unwrap();
        let location = loc_to_range(definition_loc, &self.line_col_lookup);

        let definition = get_definition_from(
            &namespace,
            &self.current_namespaces,
            &location,
        );

        // Note – is there a way to use lifetime specifiers to get rid of this and
        // just keep current namespaces as a vector of string references or something else
        // more efficient?
        self.current_namespaces.push(namespace);

        // Each time we open up a new class/module, we reset the behavioral change flag
        let previous_behavioral_change = self.behavioral_change_in_namespace;
        self.behavioral_change_in_namespace = false;

        if let Some(inner) = &node.body {
            self.visit(inner);
        }

        if self.behavioral_change_in_namespace {
            self.definitions.push(definition);
        }

        // When we're done visiting the class/module, we restore the previous behavioral change flag
        // to account for nested class/module definitions
        self.behavioral_change_in_namespace = previous_behavioral_change;

        self.current_namespaces.pop();
    }

    fn on_const(&mut self, node: &nodes::Const) {
        let Ok(name) = fetch_const_const_name(node) else {
            if let Some(s) = &node.scope {
                self.visit(s);
            }
            return;
        };

        let namespace_path = self
            .current_namespaces
            .clone()
            .into_iter()
            .filter(|namespace| namespace != &name)
            .collect::<Vec<String>>();

        self.references.push(UnresolvedReference {
            name,
            namespace_path,
            location: loc_to_range(&node.expression_l, &self.line_col_lookup),
        })
    }

    fn on_def(&mut self, node: &nodes::Def) {
        if !self.is_spec_file {
            self.behavioral_change_in_namespace = true;
        }
        lib_ruby_parser::traverse::visitor::visit_def(self, node);
    }

    fn on_defs(&mut self, node: &nodes::Defs) {
        if !self.is_spec_file {
            self.behavioral_change_in_namespace = true;
        }
        lib_ruby_parser::traverse::visitor::visit_defs(self, node);
    }
}

pub(crate) fn process_from_path(
    path: &Path,
    configuration: &Configuration,
) -> anyhow::Result<ProcessedFile> {
    let contents = file_read_contents(path, configuration)?;
    Ok(process_from_contents(contents, path, configuration))
}

pub(crate) fn process_from_contents(
    contents: String,
    path: &Path,
    configuration: &Configuration,
) -> ProcessedFile {
    let options = ParserOptions {
        buffer_name: "".to_string(),
        ..Default::default()
    };

    let lookup = LineColLookup::new(&contents);
    let parser = Parser::new(contents.clone(), options);
    let parse_result = parser.do_parse();

    let ast_option: Option<Box<Node>> = parse_result.ast;

    let ast = match ast_option {
        Some(some_ast) => some_ast,
        None => {
            return ProcessedFile {
                absolute_path: path.to_owned(),
                unresolved_references: vec![],
                definitions: vec![],
                sigils: vec![],
            }
        }
    };

    /*
       `pks` has a feature that detects a monkey patch within a module, e.g.:
       module SomeOtherPack
         some_monkey_patch
       end

       It then considers this a definition of `SomeOtherPack`. This is a bit idiosyncratic – but was intended to support experimental detection
       of monkey patches, e.g. to String.

       This causes issues for a common RSpec pattern:

       module MyModule
         RSpec.describe MyClass do
           ...
         end
       end

       To address this, we disable the monkey patch detection in spec files.
    */
    let is_spec_file = path.to_string_lossy().contains("_spec.rb")
        || path.to_string_lossy().contains("/spec/");

    let mut collector = ReferenceCollector {
        references: vec![],
        current_namespaces: vec![],
        definitions: vec![],
        line_col_lookup: lookup,
        behavioral_change_in_namespace: false,
        custom_associations: configuration.custom_associations.clone(),
        is_spec_file,
    };

    collector.visit(&ast);

    let unresolved_references = collector.references;

    let absolute_path = path.to_owned();

    // The packwerk parser uses a ConstantResolver constructed by constants inferred from the file system
    // see zeitwerk_utils for more.
    // For a parser that uses parsed constants, see the experimental parser
    let definitions = collector.definitions;

    let sigils = extract_sigils_from_contents(&contents);

    ProcessedFile {
        absolute_path,
        unresolved_references,
        definitions,
        sigils,
    }
}
