#[macro_use] extern crate nom;
use nom::IResult::Done;
#[cfg(feature = "verbose-errors")]
use nom::Err::Position;
use std::str;

named!(tag_quote, tag!("\""));
named!(unquoted<&str>, map_res!(ws!(delimited!(tag_quote, take_until!("\""),  tag_quote ) ), str::from_utf8 ) );
named!(tag_essid, ws!(tag!("ESSID:") ) );
named!(parse_essid<&str>, ws!(do_parse!( 
    tag_essid >>
    res: unquoted >>
    (res.into() )
) ) );


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_removes_quotes() {
        assert_eq!(Done(&[][..], "foobar"), unquoted(&b"\"foobar\""[..]));
    }
}
