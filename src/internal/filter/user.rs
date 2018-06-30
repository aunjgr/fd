use std::fs;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserFilter {
    uid: Check<u32>,
    gid: Check<u32>,
}

impl UserFilter {
    pub fn from_string(input: &str) -> Option<Self> {
        let mut it = input.split(':');
        let (fst, snd) = (it.next(), it.next());

        fn parse_negation(o: Option<&str>) -> Check<&str> {
            match o {
                Some("") | None => Check::Ignore,
                Some(s) if s.starts_with('!') => Check::NotEq(&s[1..]),
                Some(s) => Check::Equal(s),
            }
        }

        let uid = parse_negation(fst).and_then(|s| {
            s.parse()
                .ok()
                .or_else(|| users::get_user_by_name(s).map(|user| user.uid()))
                .or_else(|| {
                    print_error_and_exit!("Error: {} is not a recognized user name", s);
                })
        });
        let gid = parse_negation(snd).and_then(|s| {
            s.parse()
                .ok()
                .or_else(|| users::get_group_by_name(s).map(|group| group.gid()))
                .or_else(|| {
                    print_error_and_exit!("Error: {} is not a recognized group name", s);
                })
        });

        if let (Check::Ignore, Check::Ignore) = (uid, gid) {
            None
        } else {
            Some(Self { uid, gid })
        }
    }

    pub fn matches(&self, md: &fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt;

        self.uid.check(md.uid()) && self.gid.check(md.gid())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Check<T> {
    Equal(T),
    NotEq(T),
    Ignore,
}

impl<T: PartialEq> Check<T> {
    fn check(&self, v: T) -> bool {
        match *self {
            Check::Equal(ref x) => v == *x,
            Check::NotEq(ref x) => v != *x,
            Check::Ignore => true,
        }
    }
}

impl<T> Check<T> {
    fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Check<U> {
        match self {
            Check::Equal(x) => match f(x) {
                Some(r) => Check::Equal(r),
                None => Check::Ignore,
            },
            Check::NotEq(x) => match f(x) {
                Some(r) => Check::NotEq(r),
                None => Check::Ignore,
            },
            Check::Ignore => Check::Ignore,
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

    use super::Check::*;
    owner_tests! {
        empty:      ""      => None,
        uid_only:   "5"     => Some(UserFilter { uid: Equal(5), gid: Ignore    }),
        uid_gid:    "9:3"   => Some(UserFilter { uid: Equal(9), gid: Equal(3)  }),
        gid_only:   ":8"    => Some(UserFilter { uid: Ignore,   gid: Equal(8)  }),
        colon_only: ":"     => None,
        trailing:   "5:"    => Some(UserFilter { uid: Equal(5), gid: Ignore    }),

        uid_negate: "!5"    => Some(UserFilter { uid: NotEq(5), gid: Ignore    }),
        both_negate:"!4:!3" => Some(UserFilter { uid: NotEq(4), gid: NotEq(3)  }),
        uid_not_gid:"6:!8"  => Some(UserFilter { uid: Equal(6), gid: NotEq(8)  }),
    }
    //FIXME: maybe find a way to test parsing usernames ?
}
