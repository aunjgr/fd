use users::{get_user_by_name, get_group_by_name};

#[derive(Clone, Copy, Debug, PartialEq)]
enum OwnerCheck {
    Equal(u32),
    NotEq(u32),
    Ignore,
}

impl OwnerCheck {
    fn matches(&self, v: u32) -> bool {
        match *self {
            OwnerCheck::Equal(x) => x == v,
            OwnerCheck::NotEq(x) => x != v,
            OwnerCheck::Ignore => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OwnerFilter {
    user: OwnerCheck,
    group: OwnerCheck,
}

impl OwnerFilter {
    pub fn from_string(s: &str) -> Option<Self> {
        fn get_uid(name_or_id: &str) -> Option<u32> {
            name_or_id.parse().ok().or_else(|| get_user_by_name(name_or_id).map(|u| u.uid()))
        }

        fn get_gid(name_or_id: &str) -> Option<u32> {
            name_or_id.parse().ok().or_else(|| get_group_by_name(name_or_id).map(|g| g.gid()))
        }

        let mut it = s.split(':');

        let user = match it.next() {
            Some("") | None => OwnerCheck::Ignore,
            Some(v) if v.starts_with('!') => OwnerCheck::NotEq(get_uid(&v[1..])?),
            Some(v) => OwnerCheck::Equal(get_uid(v)?),
        };

        let group = match it.next() {
            Some("") | None => OwnerCheck::Ignore,
            Some(v) if v.starts_with('!') => OwnerCheck::NotEq(get_gid(&v[1..])?),
            Some(v) => OwnerCheck::Equal(get_gid(v)?),
        };

        if  (OwnerCheck::Ignore, OwnerCheck::Ignore) == (user, group) {
            None
        } else {
            Some(Self {user, group})
        }
    }

    pub fn matches(&self, uid: u32, gid: u32) -> bool {
        self.user.matches(uid) && self.group.matches(gid)
    }
}

#[cfg(test)]
mod user_parsing {
    use super::*;

    macro_rules! gen_owner_tests {
        ($($name:ident: $value:expr => $result:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let o = OwnerFilter::from_string($value);
                    assert_eq!(o, $result);
                }
            )*
        }
    }

    use super::OwnerCheck::*;
    gen_owner_tests! {
        empty:      ""      => None,
        uid_only:   "5"     => Some(OwnerFilter { user: Equal(5), group: Ignore    }),
        uid_gid:    "9:3"   => Some(OwnerFilter { user: Equal(9), group: Equal(3)  }),
        gid_only:   ":8"    => Some(OwnerFilter { user: Ignore,   group: Equal(8)  }),
        colon_only: ":"     => None,
        trailing:   "5:"    => Some(OwnerFilter { user: Equal(5), group: Ignore    }),

        uid_negate: "!5"    => Some(OwnerFilter { user: NotEq(5), group: Ignore    }),
        both_negate:"!4:!3" => Some(OwnerFilter { user: NotEq(4), group: NotEq(3)  }),
        uid_not_gid:"6:!8"  => Some(OwnerFilter { user: Equal(6), group: NotEq(8)  }),

        user_only:  "root"          => Some(OwnerFilter { user: Equal(0), group: Ignore    }),
        user_group: "!root:root"    => Some(OwnerFilter { user: NotEq(0), group: Equal(0)  }),
        group_only: ":!root"        => Some(OwnerFilter { user: Ignore,   group: NotEq(0)  }),
    }
}
