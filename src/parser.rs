use tree_sitter::Tree;

fn get_token_types(t: &Tree) {
    let mut tokens: Vec<u8> = vec![];
    let mut cursor = t.walk();
    loop {
        cursor.node().kind
    }
}
