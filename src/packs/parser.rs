use glob::glob;
use lib_ruby_parser::{Node, Parser, ParserOptions};
use rayon::prelude::*;
use std::{fs, path::PathBuf};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Reference {
    path: PathBuf,
    name: String,
}

pub fn get_references(absolute_root: PathBuf) -> Vec<Reference> {
    // Later this can come from config
    let pattern = absolute_root.join("packs/**/*.rb");

    let x = glob(pattern.to_str().unwrap())
        .expect("Failed to read glob pattern")
        .par_bridge() // Parallel iterator
        .flat_map(|entry| match entry {
            Ok(path) => extract_from_path(path),
            Err(e) => {
                println!("{:?}", e);
                panic!("blah");
            }
        })
        .collect();
    x
}

fn extract_from_path(path: PathBuf) -> Vec<Reference> {
    // TODO: This can be a debug statement instead of a print
    // println!("Now parsing {:?}", path);
    let contents = fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read contents of {}", path.to_string_lossy()));

    extract_from_contents(contents)
}

fn extract_from_contents(contents: String) -> Vec<Reference> {
    let options = ParserOptions {
        buffer_name: "".to_string(),
        ..Default::default()
    };
    let parser = Parser::new(contents, options);
    let _ret = parser.do_parse();
    // let references = vec!["test"];
    let ast = *_ret.ast.expect("No AST found!");
    println!("{}", format!("HERE I AM {:#?}", ast));

    // println!("{}", ast.inspect(2));
    // return vec![
    //     Reference {
    //         path,
    //         name: String::from("test"),
    //     }
    // ]
    return extract_from_ast(ast);
}

fn extract_from_ast(ast: Node) -> Vec<Reference> {
    match ast {
        Node::Class(x) => return extract_from_ast(*x.body.expect("no body on class node")),
        Node::Const(y) => {
            return vec![Reference {
                path: PathBuf::from("test"),
                name: y.name.clone(),
            }]
        }
        // Node::Module(z) => {
        //     match z.body {
        //         Some(b) => {
        //             return extract_from_ast(b)
        //         }
        //         None => {
        //             return vec![]
        //         }
        //     }
        // },
        Node::Module(x) => return extract_from_ast(*x.body.expect("no body on module node")),
        // Node::Alias(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::And(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::AndAsgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Arg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Args(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Array(x) => {
        //     return x.elements.iter().map(|n| extract_from_ast(n)).collect()
        // };
        // Node::ArrayPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::ArrayPatternWithTail(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::BackRef(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Begin(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Block(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Blockarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::BlockPass(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Break(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Case(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::CaseMatch(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Casgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Cbase(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Complex(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::ConstPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::CSend(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Cvar(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Cvasgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        Node::Def(x) => return extract_from_ast(*x.body.expect("no body on class node")),
        // Node::Defined(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Defs(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Dstr(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Dsym(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::EFlipFlop(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::EmptyElse(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Encoding(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Ensure(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Erange(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::False(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::File(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::FindPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Float(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::For(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::ForwardArg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::ForwardedArgs(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Gvar(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Gvasgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Hash(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::HashPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Heredoc(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::If(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::IfGuard(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::IFlipFlop(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::IfMod(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::IfTernary(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Index(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::IndexAsgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::InPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Int(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Irange(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Ivar(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Ivasgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Kwarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Kwargs(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::KwBegin(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Kwnilarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Kwoptarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Kwrestarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Kwsplat(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Lambda(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Line(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Lvar(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Lvasgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Masgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchAlt(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchAs(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchCurrentLine(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchNilPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchPattern(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchPatternP(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchRest(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchVar(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::MatchWithLvasgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Mlhs(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Next(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Nil(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::NthRef(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Numblock(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::OpAsgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Optarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Or(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::OrAsgn(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Pair(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Pin(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Postexe(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Preexe(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Procarg0(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Rational(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Redo(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Regexp(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::RegOpt(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Rescue(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::RescueBody(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Restarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Retry(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Return(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::SClass(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Selfx(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Send(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Shadowarg(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Splat(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Str(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Super(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Sym(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::True(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Undef(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::UnlessGuard(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Until(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::UntilPost(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::When(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::z => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::While(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::WhilePost(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::XHeredoc(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Xstr(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::Yield(x) => return extract_from_ast(x.body.expect("no body on class node")),
        // Node::ZSuper(x) => return extract_from_ast(x.body.expect("no body on class node")),
        _other => {
            // _other.body();
            // println!("{}", format!("HERE I AM {:#?}", _other));
            return vec![];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_path() {
        let absolute_root: PathBuf = PathBuf::from("tests/fixtures/simple_dependency_violation");
        // let references_by_file = vec![
        //     Reference { path: absolute_root.join("packs/foo/app/services/foo.rb"), name: "Bar" }
        // ];

        assert_eq!(extract_from_path(absolute_root).len(), 1);
        // assert_eq!(get_references(absolute_root), references_by_file);
        // panic!("for output...")
    }
}
