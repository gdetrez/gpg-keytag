use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while1},
    character::is_digit,
    combinator::iterator,
    IResult,
};
use std::io;

type Input<'a> = &'a [u8];
type Token<'a> = &'a [u8];

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum TokenTree<'a> {
    Node(Vec<TokenTree<'a>>),
    Leaf(Token<'a>),
}

impl<'a> From<Token<'a>> for TokenTree<'a> {
    fn from(token: Token<'a>) -> TokenTree<'a> {
        TokenTree::Leaf(token)
    }
}

pub fn deserialize(input: Input) -> anyhow::Result<TokenTree> {
    match ptree(input) {
        Ok((_, result)) => Ok(result),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => anyhow::bail!("{:?}", e),
        Err(nom::Err::Incomplete(_)) => unreachable!(),
    }
}

pub fn serialize(tree: &TokenTree, writer: &mut impl io::Write) -> io::Result<()> {
    match tree {
        TokenTree::Leaf(bytes) => {
            writer.write_all(bytes.len().to_string().as_bytes())?;
            writer.write_all(b":")?;
            writer.write_all(bytes)?;
        }
        TokenTree::Node(children) => {
            writer.write_all(b"(")?;
            for child in children {
                serialize(&child, writer)?;
            }
            writer.write_all(b")")?;
        }
    }
    Ok(())
}

fn ptree(input: Input) -> IResult<Input, TokenTree> {
    alt((pnode, pleaf))(input)
}

fn pnode(input: Input) -> IResult<Input, TokenTree> {
    let (input, _) = tag(b"(")(input)?;
    let mut it = iterator(input, ptree);
    let children: Vec<_> = it.collect();
    let (input, ()) = it.finish()?;
    let (input, _) = tag(b")")(input)?;
    Ok((input, TokenTree::Node(children)))
}

fn pleaf(input: Input) -> IResult<Input, TokenTree> {
    let (input, v) = ptoken(input)?;
    Ok((input, v.into()))
}

fn ptoken(input: Input) -> IResult<Input, Token> {
    let (input, size) = take_while1(is_digit)(input)?;
    let size: usize = std::str::from_utf8(size).unwrap().parse().unwrap();
    let (input, _) = tag(b":")(input)?;
    let (input, result) = take(size)(input)?;
    Ok((input, result))
}

#[cfg(test)]
mod tests {
    use super::{TokenTree::*, *};

    #[test]
    fn parse_leaf() {
        let input = b"12:foobarbazbizqux";
        let v = deserialize(&input[..]).unwrap();
        assert_eq!(v, Leaf(b"foobarbazbiz"));
    }

    #[test]
    fn parse_simple_node() {
        let input = b"(6:foobar)";
        let v = deserialize(&input[..]).unwrap();
        assert_eq!(v, Node(vec![Leaf(b"foobar")]));
    }

    #[test]
    fn parse_node_with_leaves() {
        let input = b"(6:foobar3:qux4:quux)";
        let v = deserialize(&input[..]).unwrap();
        assert_eq!(v, Node(vec![Leaf(b"foobar"), Leaf(b"qux"), Leaf(b"quux")]));
    }

    #[test]
    fn parse_complex_node() {
        let input = b"(6:foobar(3:qux4:quux)(2:xy))";
        let v = deserialize(&input[..]).unwrap();
        assert_eq!(
            v,
            Node(vec![
                Leaf(b"foobar"),
                Node(vec![Leaf(b"qux"), Leaf(b"quux"),]),
                Node(vec![Leaf(b"xy")])
            ])
        );
    }

    #[test]
    fn parse_node_start_with_subnode() {
        let input = b"((3:foo3:bar)3:qux(4:quux))";
        let v = deserialize(&input[..]).unwrap();
        assert_eq!(
            v,
            Node(vec![
                Node(vec![Leaf(b"foo"), Leaf(b"bar")]),
                Leaf(b"qux"),
                Node(vec![Leaf(b"quux"),]),
            ])
        );
    }

    #[test]
    fn serialize_leaf() {
        let mut output: Vec<u8> = Vec::new();
        serialize(&Leaf(b"foobar"), &mut output).unwrap();
        assert_eq!(&output, b"6:foobar");
    }
    #[test]
    fn serialize_node() {
        let mut output: Vec<u8> = Vec::new();
        serialize(&Node(vec![]), &mut output).unwrap();
        assert_eq!(&output, b"()");
    }
    #[test]
    fn serialize_node_with_children() {
        let mut output: Vec<u8> = Vec::new();
        serialize(
            &Node(vec![
                Leaf(b"foobar"),
                Node(vec![Leaf(b"comment"), Leaf(b"qux")]),
            ]),
            &mut output,
        )
        .unwrap();
        assert_eq!(&output, b"(6:foobar(7:comment3:qux))");
    }
}
