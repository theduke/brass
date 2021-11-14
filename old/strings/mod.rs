// mod interner;

use std::rc::Rc;

/// An abstraction over string types.
///
/// This type makes it possible to work with strings more efficiently.
pub struct Str(Repr);

enum Repr {
    /// A static string.
    Static(&'static str),
    /// Inlined small string.
    Small(SmallStr),
    /// A heap allocated string.
    String(String),
    /// A shared string that is behind an Rc<_> and can be cheaply cloned.
    // TODO: add a new, public SharedStr type so users can represent shared
    // strings in the type system.
    Shared(Rc<str>),
}

impl Str {
    pub fn from_str(value: &str) -> Self {
        SmallStr::from_str(value)
    }

    pub fn new() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        Self(Repr::Small(SmallStr {
            len: 0,
            data: Default::default(),
        }))
    }

    pub fn shared(value: impl Into<Rc<str>>) -> Self {
        Self(Repr::Shared(value.into()))
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Repr::Static(s) => s.len(),
            Repr::Small(s) => s.len(),
            Repr::String(s) => s.len(),
            Repr::Shared(s) => s.len(),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Repr::Static(s) => s,
            Repr::Small(s) => s.as_str(),
            Repr::String(s) => &s,
            Repr::Shared(s) => &s,
        }
    }

    pub fn stat(value: &'static str) -> Self {
        Self(Repr::Static(value))
    }

    pub fn into_shared(self) -> Self {
        match self.0 {
            Repr::Static(_) => self,
            Repr::Small(s) => Self(Repr::Shared(s.as_str().into())),
            Repr::String(s) => Self(Repr::Shared(s.into())),
            Repr::Shared(_) => self,
        }
    }

    pub fn push(self, c: char) -> Self {
        match self.0 {
            Repr::Static(s) => SmallStr::from_str_and_char(s, c),
            Repr::Small(s) => s.push(c),
            Repr::String(mut s) => {
                s.push(c);
                Self(Repr::String(s))
            }
            Repr::Shared(s) => {
                // TODO: avoid redundant size check
                if s.len() + c.len_utf8() <= SmallStr::CAP {
                    SmallStr::from_str(&s)
                } else {
                    let mut new = s.to_string();
                    new.push(c);
                    Self(Repr::String(new))
                }
            }
        }
    }

    pub fn push_str(self, value: &str) -> Self {
        match self.0 {
            Repr::Static(s) => {
                if s.len() + value.len() < SmallStr::CAP {
                    SmallStr::from_str(s).push_str(value)
                } else {
                    let mut new = String::with_capacity(s.len() + value.len());
                    new.push_str(s);
                    new.push_str(value);
                    Self(Repr::String(new))
                }
            }
            Repr::Small(s) => s.push_str(value),
            Repr::String(mut s) => {
                s.push_str(value);
                Self(Repr::String(s))
            }
            Repr::Shared(s) => {
                if s.len() + value.len() < SmallStr::CAP {
                    // TODO: avoid redundant size check
                    SmallStr::from_str(&s).push_str(value)
                } else {
                    let mut new = s.to_string();
                    new.push_str(value);
                    Self(Repr::String(new))
                }
            }
        }
    }
}

impl Default for Str {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::ops::Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            Repr::Static(s) => s,
            Repr::Small(s) => s.as_str(),
            Repr::String(s) => &s,
            Repr::Shared(s) => &s,
        }
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        match &self.0 {
            Repr::Static(s) => s,
            Repr::Small(s) => s.as_str(),
            Repr::String(s) => &s,
            Repr::Shared(s) => s.as_ref(),
        }
    }
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Repr::Static(s) => s.fmt(f),
            Repr::Small(s) => s.as_str().fmt(f),
            Repr::String(s) => s.fmt(f),
            Repr::Shared(s) => s.fmt(f),
        }
    }
}

impl Clone for Str {
    fn clone(&self) -> Self {
        Self(match &self.0 {
            Repr::Static(s) => Repr::Static(s),
            Repr::Small(s) => Repr::Small(s.clone()),
            Repr::String(s) => Repr::String(s.clone()),
            Repr::Shared(s) => Repr::Shared(s.clone()),
        })
    }
}

impl PartialEq for Str {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Repr::Shared(s1), Repr::Shared(s2)) => Rc::ptr_eq(s1, s2),
            (Repr::Small(s1), Repr::Small(s2)) => s1.len == s2.len && s1.data == s2.data,
            (Repr::Static(s1), Repr::Static(s2)) => s1.as_ptr() == s2.as_ptr(),
            (Repr::Static(s1), Repr::Small(s2)) => *s1 == s2.as_str(),
            (Repr::Static(s1), Repr::String(s2)) => *s1 == s2,
            (Repr::Static(s1), Repr::Shared(s2)) => (*s1).eq(s2.as_ref()),
            (Repr::Small(s1), Repr::Static(s2)) => s1.as_str() == *s2,
            (Repr::Small(s1), Repr::String(s2)) => s1.as_str() == s2,
            (Repr::Small(s1), Repr::Shared(s2)) => s1.as_str() == s2.as_ref(),
            (Repr::String(s1), Repr::Static(s2)) => s1 == *s2,
            (Repr::String(s1), Repr::Small(s2)) => s1 == s2.as_str(),
            (Repr::String(s1), Repr::String(s2)) => s1 == s2,
            (Repr::String(s1), Repr::Shared(s2)) => s1 == s2.as_ref(),
            (Repr::Shared(s1), Repr::Static(s2)) => s1.as_ref().eq(*s2),
            (Repr::Shared(s1), Repr::Small(s2)) => s1.as_ref() == s2.as_str(),
            (Repr::Shared(s1), Repr::String(s2)) => s1.as_ref() == s2.as_str(),
        }
    }
}

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for Str {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<'a> PartialEq<Str> for &'a str {
    fn eq(&self, other: &Str) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<String> for Str {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Str> for String {
    fn eq(&self, other: &Str) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Str {}

impl PartialOrd for Str {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for Str {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::hash::Hash for Str {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a> From<&'a str> for Str {
    #[inline]
    fn from(v: &'a str) -> Self {
        SmallStr::from_str(v)
    }
}

impl<'a> From<&'a String> for Str {
    #[inline]
    fn from(v: &'a String) -> Self {
        SmallStr::from_str(v)
    }
}

impl From<Rc<str>> for Str {
    #[inline]
    fn from(s: Rc<str>) -> Self {
        Self(Repr::Shared(s))
    }
}

impl From<String> for Str {
    #[inline]
    fn from(s: String) -> Self {
        Self(Repr::String(s))
    }
}

// SmallStr.

#[derive(Clone)]
struct SmallStr {
    len: u8,
    data: [u8; Self::CAP],
}

impl SmallStr {
    const CAP: usize = 22;

    fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { &*std::ptr::slice_from_raw_parts(self.data.as_ptr(), self.len as usize) }
    }

    // #[inline]
    // fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
    //     unsafe {
    //         &mut *(std::ptr::slice_from_raw_parts(self.data.as_ptr(), self.len as usize)
    //             as *mut [u8])
    //     }
    // }

    #[inline]
    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice()) }
    }

    fn from_str(value: &str) -> Str {
        let len = value.len();
        if len <= Self::CAP {
            // TODO: must this be done more efficiently or does the optimizer take care of it anyway?
            let mut data: [u8; Self::CAP] = Default::default();
            let mut index = 0;
            for byte in value.as_bytes() {
                data[index] = *byte;
                index += 1;
            }
            Str(Repr::Small(Self {
                len: len as u8,
                data,
            }))
        } else {
            Str(Repr::String(value.to_string()))
        }
    }

    fn from_str_and_char(value: &str, c: char) -> Str {
        let mut value_len = value.len();
        let len = value_len + c.len_utf8();

        if len <= Self::CAP {
            // TODO: check assembly if this code needs to be smarter or if the optimizer takes care of it.
            let mut data: [u8; Self::CAP] = Default::default();
            for (index, byte) in value.as_bytes().into_iter().enumerate() {
                data[index] = *byte;
            }

            for b in c.encode_utf8(&mut [0u8; 4]).as_bytes() {
                data[value_len] = *b;
                value_len += 1;
            }

            Str(Repr::Small(Self {
                len: len as u8,
                data,
            }))
        } else {
            let mut s = String::with_capacity(len);
            s.push_str(value);
            s.push(c);
            Str(Repr::String(s))
        }
    }

    fn push(mut self, c: char) -> Str {
        if c.len_utf8() == 1 && self.len < Self::CAP as u8 {
            self.data[self.len as usize] = c as u8;
            self.len += 1;
            Str(Repr::Small(self))
        } else {
            // TODO: check assembyl for efficiency...
            self.push_str(c.encode_utf8(&mut [0u8; 4]))
        }
    }

    fn push_str(mut self, s: &str) -> Str {
        let mut index = self.len as usize;
        let new_len = index + s.len();
        if new_len <= Self::CAP {
            for byte in s.as_bytes() {
                self.data[index] = *byte;
                index += 1;
            }
            self.len = index as u8;
            Str(Repr::Small(self))
        } else {
            let mut new = String::with_capacity(new_len);
            new.push_str(self.as_str());
            new.push_str(s);
            Str(Repr::String(new))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_small(s: &Str) -> &SmallStr {
        match &s.0 {
            Repr::Small(s) => s,
            _ => panic!("expected a Repr::Small"),
        }
    }

    fn as_string(s: &Str) -> &String {
        match &s.0 {
            Repr::String(s) => s,
            _ => panic!("expected a Repr::String"),
        }
    }

    #[test]
    fn test_str_basic() {
        let s1 = Str::from("a");
        assert_eq!(s1, "a");
        assert_eq!(s1.len(), 1);
        assert_eq!(as_small(&s1).len, 1);
        assert_eq!(
            as_small(&s1).data,
            [b'a', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        // Push single char.
        let s2 = s1.push('b');
        assert_eq!(s2.len(), 2);
        assert_eq!(s2, "ab");
        assert_eq!(as_small(&s2).len, 2);
        assert_eq!(
            as_small(&s2).data,
            [b'a', b'b', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        // Push multi-byte char.
        let s3 = s2.push('ü');
        assert_eq!(s3.len(), 4);
        assert_eq!(s3, "abü");
        assert_eq!(as_small(&s3).len, 4);
        assert_eq!(
            as_small(&s3).data,
            [b'a', b'b', 195, 188, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        // Full SmallStr.
        let full_small = Str::from("0000000000000000000000");
        assert_eq!(full_small.len(), SmallStr::CAP);
        assert_eq!(as_small(&full_small).len, SmallStr::CAP as u8);

        // Push single char to full SmallStr.
        let ss = full_small.clone().push('a');
        assert_eq!(ss.len(), 23);
        assert_eq!(ss, "0000000000000000000000a");
        as_string(&ss);

        // Push multibyte char to full SmallStr.
        let ss = full_small.clone().push('ü');
        assert_eq!(ss.len(), 24);
        assert_eq!(ss, "0000000000000000000000ü");
        as_string(&ss);

        // Push str to full SmallStr.
        let ss = full_small.clone().push_str("abcde");
        assert_eq!(ss.len(), 27);
        assert_eq!(ss, "0000000000000000000000abcde");
        as_string(&ss);

        // Repr::String
        let ss = Str::from("hello there".to_string());
        as_string(&ss);
        assert_eq!(ss, "hello there");

        // Repr::Shared
        let ss = Str::from(Rc::<str>::from("hello there".to_string()));
        assert_eq!(ss, ss);
        assert_eq!(ss, "hello there");
    }

    #[test]
    fn test_str_eq() {
        let sstatic = Str::stat("hello there");
        let ssmall = Str::from("hello there");
        let sstring = Str::from("hello there".to_string());
        let sshared = Str::from("hello there").into_shared();

        assert_eq!(sstatic, "hello there");
        assert_eq!(ssmall, "hello there");
        assert_eq!(sstring, "hello there");
        assert_eq!(sshared, "hello there");

        assert_eq!(sstatic, ssmall);
        assert_eq!(sstatic, sstring);
        assert_eq!(sstatic, sshared);
    }
}
