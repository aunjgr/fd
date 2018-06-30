use std::fs;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserFilter {
    uid: Check<u32>,
    gid: Check<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Check<T> {
    Equal(T),
    NotEq(T),
    Ignore,
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
            (s, _) => s
                .parse()
                .ok()
                .or_else(|| users::get_user_by_name(s).map(|user| user.uid()))
                .or_else(|| {
                    print_error_and_exit!("Error: {} is not a recognized user name", s);
                }),
        };
        let gid = match snd {
            Some("") | None => None,
            Some(s) => s
                .parse()
                .ok()
                .or_else(|| users::get_group_by_name(s).map(|group| group.gid()))
                .or_else(|| {
                    print_error_and_exit!("Error: {} is not a recognized group name", s);
                }),
        };

        use self::Check::*;
        let uid = match uid {
            Some(u) => Equal(u),
            _ => Ignore,
        };
        let gid = match gid {
            Some(g) => Equal(g),
            _ => Ignore,
        };

        if let (Ignore, Ignore) = (uid, gid) {
            None
        } else {
            Some(Self { uid, gid })
        }
    }

    pub fn matches(&self, md: &fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt;

        let uid_ok = match self.uid {
            Check::Equal(u) => u == md.uid(),
            _ => true,
        };
        let gid_ok = match self.gid {
            Check::Equal(g) => g == md.gid(),
            _ => true,
        };

        uid_ok && gid_ok
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

    use super::Check::*;
    owner_tests! {
        empty:      ""      => None,
        uid_only:   "5"     => Some(UserFilter { uid: Equal(5), gid: Ignore    }),
        uid_gid:    "9:3"   => Some(UserFilter { uid: Equal(9), gid: Equal(3)  }),
        gid_only:   ":8"    => Some(UserFilter { uid: Ignore,   gid: Equal(8)  }),
        colon_only: ":"     => None,
        trailing:   "5:"    => Some(UserFilter { uid: Equal(5), gid: Ignore    }),
    }
    //FIXME: maybe find a way to test parsing usernames ?
}
