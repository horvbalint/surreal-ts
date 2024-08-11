use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::combinator::rest;
use nom::sequence::pair;
use nom::{
    bytes::complete::{is_not, take_until1},
    combinator::opt,
    sequence::delimited,
    IResult,
};

#[derive(Debug)]
pub struct FieldProps<'a> {
    pub is_optional: bool,
    pub is_array: bool,
    pub is_record: bool,
    pub name: &'a str,
}

pub fn parse_type(input: &str) -> IResult<&str, FieldProps> {
    let (input, inner) = opt(delimited(tag("option<"), parse_type, tag(">")))(input)?;
    if let Some(props) = inner {
        return Ok((
            input,
            FieldProps {
                is_optional: true,
                ..props
            },
        ));
    }

    let (input, inner) = opt(delimited(
        tag("array<"),
        pair(parse_type, opt(is_not(">"))),
        tag(">"),
    ))(input)?;
    if let Some((props, _)) = inner {
        return Ok((
            input,
            FieldProps {
                is_array: true,
                ..props
            },
        ));
    }

    let (input, inner) = opt(delimited(
        tag("set<"),
        pair(parse_type, opt(is_not(">"))),
        tag(">"),
    ))(input)?;
    if let Some((props, _)) = inner {
        return Ok((
            input,
            FieldProps {
                is_array: true,
                ..props
            },
        ));
    }

    let (input, inner) = opt(delimited(tag("record<"), is_not(">"), tag(">")))(input)?;
    if let Some(reference) = inner {
        return Ok((
            input,
            FieldProps {
                name: reference,
                is_record: true,
                is_array: false,
                is_optional: false,
            },
        ));
    }

    let (input, name) = alpha1(input)?;
    Ok((
        input,
        FieldProps {
            name,
            is_record: false,
            is_array: false,
            is_optional: false,
        },
    ))
}

pub fn parse_comment(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = opt(take_until1("COMMENT"))(input)?;
    let (input, res) = opt(tag("COMMENT "))(input)?;

    if res.is_some() {
        let (input, comment) = delimited(tag("'"), is_not("'"), tag("'"))(input)?;
        Ok((input, Some(comment)))
    } else {
        Ok((input, None))
    }
}

pub fn parse_type_from_definition(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_until1("TYPE")(input)?;
    let (input, _) = tag("TYPE ")(input)?;
    let (input, raw_type) = alt((
        take_until1("COMMENT"),
        take_until1("DEFAULT"),
        take_until1("VALUE"),
        take_until1("ASSERT"),
        take_until1("PERMISSIONS"),
        rest,
    ))(input)?;

    Ok((input, raw_type))
}
