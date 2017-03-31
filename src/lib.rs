// Copyright 2016 Mark Sta Ana.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

#[macro_use]
extern crate nom;
use nom::IResult::Done;
#[cfg(feature = "verbose-errors")]
use nom::Err::Position;
use std::str;
use helpers::buf_to_i32;

/// Wifi struct used to return information about wifi hotspots
/// Warning: parser for security has not been implemented!
#[derive(Debug,PartialEq,Eq)]
pub struct Wifi {
    /// mac address
    pub mac: String,
    /// hotspot name
    pub ssid: String,
    pub channel: String,
    pub signal_level: String,
    /// this field is currently empty in the Linux version of the lib
    pub security: String,
}

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
#[allow(dead_code)]
named!(parse_address<&str>, 
    do_parse!( 
            take!(20) >>
            tag_address >>
            res: map_res!(nom::not_line_ending, str::from_utf8) >>
            ( res.into() )
        ) );

#[allow(dead_code)]
named!(tag_channel, delimited!(
            char!('('),
            preceded!(tag!("Channel "), nom::digit),
            char!(')')
));

#[allow(dead_code)]
named!(parse_channel<&str>, do_parse!( // should probably be an int
    take_until!("(") >> // junk
    res : map_res!(tag_channel, str::from_utf8) >>
    ( res.into() )
) );

#[allow(dead_code)]
named!(tag_signal, tag!("Signal level") );
#[allow(dead_code)]
named!(tag_signal_value, delimited!(
    tag!("="), 
    take_until!("/"),
    tag!("/")
    ) );

#[allow(dead_code)]
mod helpers {
    // shameless stolen from Jan-Erik Rediger's excellent https://github.com/badboy/iso8601
    use std::str::{FromStr, from_utf8_unchecked};

    pub fn to_string(s: &[u8]) -> &str {
        unsafe { from_utf8_unchecked(s) }
    }
    pub fn to_i32(s: &str) -> i32 {
        FromStr::from_str(s).unwrap()
    }
    pub fn to_u32(s: &str) -> u32 {
        FromStr::from_str(s).unwrap()
    }

    pub fn buf_to_u32(s: &[u8]) -> u32 {
        to_u32(to_string(s))
    }
    pub fn buf_to_i32(s: &[u8]) -> i32 {
        to_i32(to_string(s))
    }
}

#[allow(dead_code)]    
named!(parse_signal_strength_ubuntu<i32>, ws!(do_parse!( 
    tag_signal >> 
    res: tag_signal_value >>
    take!(3) >> // junk
    ( buf_to_i32(res) )
) ) );

fn parse_signal_strength(value : i32) -> String { // original Wifi struct expect a string, using i32 would be a breaking change
    calc_decibels(value).to_string()
}

#[allow(dead_code)]
fn calc_decibels(raw : i32) -> i32 {
    ((100 * raw) / 100) / 2 - 100
}

named!(wifi<Wifi>, do_parse!( 
            mac : parse_address >>
            ssid : parse_essid >>
            take!(73) >> // FIXME: probably a better way to do this
            channel : parse_channel >>
            take!(449) >> // FIXME: probably a better way to do this
            signal_num : parse_signal_strength_ubuntu >> // FIXME: hard coded for ubuntu output
            ( Wifi { 
                mac: mac.into(), 
                ssid: ssid.into(), 
                channel: channel.into(), 
                signal_level: parse_signal_strength(signal_num), 
                security: "".to_string() } )            
        ) );

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
        assert_eq!(Done(&[][..], "00:35:1A:6F:0F:40"), parse_address(&b"          Cell 01 - Address: 00:35:1A:6F:0F:40"[..]) );
    }

    #[test]
    fn it_finds_the_channel() {
        assert_eq!(Done(&[][..], "6"), parse_channel(&b"                    Frequency:2.437 GHz (Channel 6)"[..]) );
    }

    #[test]
    fn it_finds_the_signal_strength_for_ubuntu() {
        assert_eq!(Done(&[][..], 56), parse_signal_strength_ubuntu(&b"                    Signal level=56/100"[..]) );
    }

    #[test]
    fn it_computes_rssi() {
        assert_eq!(calc_decibels(56), -72);
    }

    #[test]
    fn it_converts_signal_strength_to_rssi() {
        if let Done(_,signal_strength) = parse_signal_strength_ubuntu(&b"                    Signal level=56/100"[..]) {
            assert_eq!(calc_decibels(signal_strength), -72 );
        } else {
            println!("Failed to parse signal strength!");
            assert_eq!(1, 0); // FIXME: this is messy way of error handling
        }        
    }

    #[test]
    fn it_gets_wifi() {
        let cell = br#"          Cell 01 - Address: 00:35:1A:6F:0F:40
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
                    Signal level=56/100"#;

        if let Done(_, res) = wifi(&cell[..]) {
            assert_eq!( Wifi { 
                                mac: "00:35:1A:6F:0F:40".to_string(), 
                                ssid: "TEST-Wifi".to_string(), 
                                channel: "6".to_string(), 
                                signal_level: "-72".to_string(), 
                                security: "".to_string() 
                            }, res );
            println!("Woot!");
        } else {
            println!("Failed to parse wifi!");
            assert_eq!(1,0);
        }
    }
}




/* useful snippets
// TODO turn this into a helper func for debugging
if let Done(_,res) = tag_address(&b"Address: 00:35:1A:6F:0F:40"[..]) {
    println!("Hai! {:?}", str::from_utf8(res).unwrap() ) ;
} else {
    println!("nope");
}

    fn it_gets_raw_over_bytes() {
        let raw = br#"123
hello "world"
def"#;
        let byte = b"123\nhello \"world\"\ndef";
        
        assert_eq!(raw, byte);
        println!("raw:\n{}\nbytes:\n{}",str::from_utf8(raw).unwrap(), str::from_utf8(byte).unwrap() );
        assert_eq!(1,0);
    }


*/

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
