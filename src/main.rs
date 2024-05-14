mod keyfile;
use keyfile::TokenTree;

const GPG_COMMENT_FIELD: &[u8] = b"comment";

const USAGE: &str = r#"gpg-keytags: add a comment to your GPG key.

Usage:

    gpg-keytag <keyfile> [<comment>]

You must specify the path to a private key file (usually in .gnupg/private-keys-v1.d/). If only
<keyfile> is given, gpg-keytag will print the current tag. If <comment> is given as well,
gpg-keytag will replace the current tag with the new comment.
"#;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, keyfile] => {
            let content = std::fs::read(keyfile)?;
            let tree = keyfile::deserialize(&content)?;
            println!("{}", get_comment(&tree).as_deref().unwrap_or("(none)"));
        }
        [_, keyfile, comment] => {
            let content = std::fs::read(keyfile)?;
            let mut tree = keyfile::deserialize(&content)?;
            upsert_comment(&mut tree, &comment);
            let mut writer = std::fs::File::create(keyfile)?;
            keyfile::serialize(&tree, &mut writer)?;
        }
        _ => eprintln!("{}", USAGE),
    }
    Ok(())
}

fn get_comment(tt: &TokenTree) -> Option<String> {
    use keyfile::TokenTree::*;
    let values = if let Node(children) = tt {
        children
    } else {
        return None;
    };
    for value in values.iter().skip(1) {
        match value {
            Node(xs) if xs.get(0) == Some(&Leaf(GPG_COMMENT_FIELD)) => match xs.get(1) {
                Some(Leaf(bs)) => return Some(String::from_utf8_lossy(bs).to_string()),
                _ => return None,
            },
            _ => {}
        }
    }
    None
}

fn upsert_comment<'a>(tt: &mut TokenTree<'a>, value: &'a str) {
    use keyfile::TokenTree::*;
    let children = if let Node(children) = tt {
        children
    } else {
        return;
    };
    // Look for an existing comment and replace it with the new one
    for child in children.iter_mut() {
        match child {
            Node(xs) if xs.get(0) == Some(&Leaf(GPG_COMMENT_FIELD)) => {
                *xs = vec![Leaf(GPG_COMMENT_FIELD), Leaf(value.as_bytes())];
                return;
            }
            _ => {}
        }
    }
    // Didn't find a comment, insert at the end
    children.push(Node(vec![Leaf(GPG_COMMENT_FIELD), Leaf(value.as_bytes())]));
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyfile::TokenTree::*;

    #[test]
    fn get_comment_exist() {
        let tree = TokenTree::Node(vec![
            TokenTree::Leaf(b"private-key"),
            TokenTree::Node(vec![
                TokenTree::Leaf(b"notcomment"),
                TokenTree::Leaf(b"foobar"),
            ]),
            TokenTree::Leaf(b"somevalue"),
            TokenTree::Node(vec![
                TokenTree::Leaf(b"comment"),
                TokenTree::Leaf(b"foobar"),
            ]),
        ]);
        assert_eq!(get_comment(&tree), Some(String::from("foobar")));
    }

    #[test]
    fn get_comment_missing_value() {
        let tree = TokenTree::Node(vec![
            TokenTree::Leaf(b"private-key"),
            TokenTree::Node(vec![TokenTree::Leaf(b"comment")]),
        ]);
        assert_eq!(get_comment(&tree), None);
    }

    #[test]
    fn get_comment_missing() {
        let tree = TokenTree::Node(vec![
            TokenTree::Leaf(b"private-key"),
            TokenTree::Node(vec![
                TokenTree::Leaf(b"notcomment"),
                TokenTree::Leaf(b"foobar"),
            ]),
        ]);
        assert_eq!(get_comment(&tree), None);
    }

    #[test]
    fn get_comment_leaf() {
        let tree = TokenTree::Leaf(b"qux");
        assert_eq!(get_comment(&tree), None);
    }

    #[test]
    fn upsert_comment_insert() {
        let mut tree = Node(vec![
            Leaf(b"private-key"),
            Node(vec![Leaf(b"notcomment")]),
            Leaf(b"otherthing"),
        ]);
        upsert_comment(&mut tree, "foobar");
        assert_eq!(
            tree,
            Node(vec![
                Leaf(b"private-key"),
                Node(vec![Leaf(b"notcomment"),]),
                Leaf(b"otherthing"),
                Node(vec![Leaf(b"comment"), Leaf(b"foobar"),]),
            ])
        );
    }

    #[test]
    fn upsert_comment_update() {
        let mut tree = Node(vec![
            Leaf(b"private-key"),
            Node(vec![Leaf(b"comment"), Leaf(b"quux")]),
            Leaf(b"otherthing"),
        ]);
        upsert_comment(&mut tree, "foobar");
        assert_eq!(
            tree,
            Node(vec![
                Leaf(b"private-key"),
                Node(vec![Leaf(b"comment"), Leaf(b"foobar")]),
                Leaf(b"otherthing"),
            ])
        );
    }
}
