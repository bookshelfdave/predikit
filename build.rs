//use peginator_codegen::Compile;

fn main() {
    let out = format!("./src/predikit/comp/pkparser.rs");

    peginator_codegen::Compile::file("./src/predikit/comp/pkparser.ebnf")
        .destination(out)
        .format()
        // make unused code warnings go away
        .prefix("#![allow(dead_code)]".to_owned())
        .run_exit_on_error();

    println!("cargo:rerun-if-changed=./src/predikit/comp/pkparser.ebnf");
}
