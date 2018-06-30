#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserFilter {
    uid: Option<u32>,
    gid: Option<u32>,
}

impl UserFilter {
    pub fn from_string(input: &str) -> Option<Self> {
        let colon = input.find(':');
        let (fst, snd) = match colon {
            Some(p) => {
                let (a, b) = input.split_at(p);
                (a, Some(&b[1..]))
            }
            _ => (input, None),
        };

        let uid = match (fst, colon) {
            ("", None) => return None, // can't have only empty uid
            ("", Some(_)) => None,     // empty uid is ok when we have colon
            (s, _) => Some(s.parse().ok()?),
        };
        let gid = match snd {
            Some("") | None => None,
            Some(s) => Some(s.parse().ok()?),
        };

        if uid.is_none() && gid.is_none() {
            None
        } else {
            Some(Self { uid, gid })
        }
    }
}

#[cfg(test)]
mod user_parsing {
    use super::*;

    macro_rules! owner_tests {
        ($($name:ident: $value:expr => $result:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let o = UserFilter::from_string($value);
                    assert_eq!(o, $result);
                }
            )*
        };
    }

    owner_tests! {
        empty:      ""      => None,
        uid_only:   "5"     => Some(UserFilter { uid: Some(5), gid: None   }),
        uid_gid:    "9:3"   => Some(UserFilter { uid: Some(9), gid: Some(3)}),
        gid_only:   ":8"    => Some(UserFilter { uid: None,    gid: Some(8)}),
        colon_only: ":"     => None,
        trailing:   "5:"    => Some(UserFilter { uid: Some(5), gid: None   }),
    }
}

