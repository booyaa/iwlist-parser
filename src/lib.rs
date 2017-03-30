#[macro_use] extern crate nom;
use nom::IResult::Done;
#[cfg(feature = "verbose-errors")]
use nom::Err::Position;
use std::str;

#[allow(dead_code)]
named!(tag_quote, tag!("\""));
#[allow(dead_code)]
named!(unquoted<&str>, map_res!(ws!(delimited!(tag_quote, take_until!("\""),  tag_quote ) ), str::from_utf8 ) );

#[allow(dead_code)]
named!(tag_essid, ws!(tag!("ESSID:") ) );
#[allow(dead_code)]
named!(parse_essid<&str>, ws!(do_parse!( 
    tag_essid >>
    res: unquoted >>
    (res.into() )
) ) );

#[allow(dead_code)]
named!(tag_address, ws!(tag!("Address: ") ) );
named!(parse_address<&str>, 
    do_parse!( 
            take!(20) >>
            tag_address >>
            res: map_res!(nom::not_line_ending, str::from_utf8) >>
            ( res.into() )
        ) );

// parse_address: take(20) to glob [          Cell 01 - ]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_removes_quotes() {
        assert_eq!(Done(&[][..], "foobar"), unquoted(&b"\"foobar\""[..]) );
    }

    #[test]
    fn it_finds_the_ssid() {
        assert_eq!(Done(&[][..], "foobar"), parse_essid(&b"                    ESSID:\"foobar\""[..]) );
    }

    #[test]
    fn it_finds_the_address() {
        // TODO turn this into a helper func for debugging
        // if let Done(_,res) = tag_address(&b"Address: 00:35:1A:6F:0F:40"[..]) {
        //     println!("Hai! {:?}", str::from_utf8(res).unwrap() ) ;
        // } else {
        //     println!("nope");
        // }
        
        assert_eq!(Done(&[][..], "00:35:1A:6F:0F:40"), parse_address(&b"          Cell 01 - Address: 00:35:1A:6F:0F:40"[..]) );
    }
}



/* sample hotspot
          Cell 01 - Address: 00:35:1A:6F:0F:40
                    ESSID:"TEST-Wifi"
                    Protocol:IEEE 802.11gn
                    Mode:Master
                    Frequency:2.437 GHz (Channel 6)
                    Encryption key:on
                    Bit Rates:6 Mb/s; 9 Mb/s; 12 Mb/s; 18 Mb/s; 24 Mb/s
                              36 Mb/s; 48 Mb/s; 54 Mb/s
                    Extra:rsn_ie=30140100000fac040100000fac040100000fac022800
                    IE: IEEE 802.11i/WPA2 Version 1
                        Group Cipher : CCMP
                        Pairwise Ciphers (1) : CCMP
                        Authentication Suites (1) : PSK
                    Signal level=56/100
*/