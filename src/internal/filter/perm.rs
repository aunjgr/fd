#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PermissionFilter {
    Exact(u32, u32),
    All(u32),
    Any(u32),

    NotExact(u32, u32),
    NotAll(u32),
    NotAny(u32),
}

impl PermissionFilter {
    pub fn from_string(s: &str) -> Option<Self> {
        let mut bytes = s.as_bytes();

        let negated = match bytes.first()? {
            &b'!' => { bytes = &bytes[1..]; true },
            _ => false,
        };

        let mode = match bytes.first()? {
            byte if b"=-/".contains(byte) => { bytes = &bytes[1..]; *byte },
            _ => b'=',
        };

        if !(3..=4).contains(&bytes.len()) {
            return None;
        }

        let (mut bits, mut mask) = (0u32, 0u32);
        for byte in bytes {
            bits <<= 3;
            mask <<= 3;

            match byte {
                &b'.' => (),
                digit @ b'0'..=b'7' => { bits += (digit - b'0') as u32; mask += 7; },
                _ => return None,
            }
        }

        match (negated, mode) {
            (false, b'=') => Some(Self::Exact(bits, mask)),
            (false, b'-') => Some(Self::All(bits)),
            (false, b'/') => Some(Self::Any(bits)),

            (true, b'=') => Some(Self::NotExact(bits, mask)),
            (true, b'-') => Some(Self::NotAll(bits)),
            (true, b'/') => Some(Self::NotAny(bits)),

            _ => unreachable!(),
        }
    }

    pub fn matches(&self, perm: u32) -> bool {
        match *self {
            PermissionFilter::Exact(bits, mask) => perm & mask == bits,
            PermissionFilter::All(bits) => bits & perm == bits,
            PermissionFilter::Any(bits) => bits == 0 || bits & perm != 0,

            PermissionFilter::NotExact(bits, mask) => perm & mask != bits,
            PermissionFilter::NotAll(bits) => bits & perm != bits,
            PermissionFilter::NotAny(bits) => bits != 0 && bits & perm == 0,
        }
    }
}
