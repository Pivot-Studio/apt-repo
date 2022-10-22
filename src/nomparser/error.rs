use nom::{
    branch::alt,
    bytes::complete::is_not,
    combinator::*,
    error::{FromExternalError, ParseError},
    IResult, InputTake,
};

use crate::ast::{
    node::{error::ErrorNode, Node},
    range::Range,
};

use super::{box_node, Span};

pub fn except<'a, E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::fmt::Error>>(
    except: &'static str,
    msg: &'static str,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Box<dyn Node>, E> {
    move |i| {
        let (mut i, sp) = recognize(is_not(Span::from(except)))(i)?;
        let mut r = vec![];
        for c in sp.chars() {
            if c == '{' {
                r.push('}');
            }
            if c == '}' {
                r.pop();
            }
        }
        let mut src = sp.fragment().to_string();
        let mut next;
        while r.len() > 0 {
            (i, next) = i.take_split(1);
            let nextch = next.fragment().chars().next().unwrap();
            src.push(next.fragment().chars().next().unwrap());
            if nextch == '{' {
                r.push('}');
            }
            if nextch == '}' {
                r.pop();
            }
        }
        let msg = msg.to_string();
        let end = sp.take_split(sp.len()).0;
        let node = box_node(ErrorNode {
            msg,
            src,
            range: Range::new(sp, end),
        });
        Ok((i, node))
    }
}

pub fn alt_except<'a, E, F>(
    parser: F,
    ex: &'static str,
    msg: &'static str,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Box<dyn Node>, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::fmt::Error>,
    F: FnMut(Span<'a>) -> IResult<Span<'a>, Box<dyn Node>, E>,
{
    alt((parser, except(ex, msg)))
}

// pub fn expect<'a, E,F>(
//     parser: F,
//     ex: &'static str,
//     msg: &'static str,
// ) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Box<dyn Node>, E>
// where E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::fmt::Error>,
//       F: FnMut(Span<'a>) -> IResult<Span<'a>, Box<dyn Node>, E>, {
//     alt((
//         parser,
//         move |i:Span<'a>|->IResult<Span, Box<dyn Node>, E>{
//             let msg = msg.to_string();
//             let src = i.fragment().to_string();
//             let (i,out) = recognize(is_not(Span::from(ex)))(i)?;
//             let end = i.take_split(out.len()).0;
//             let node = box_node(ErrorNode {
//                 msg,
//                 src,
//                 range: Range::new(i, end),
//             });
//             Ok((i,node))
//         },
//     ))
// }