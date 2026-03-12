#![no_std]

use core::ffi::c_char;

#[repr(C)]
pub struct IDEISProjectManifest {
    pub id: [u8; 32],
    pub name: [u8; 64],
    pub repos: [[u8; 64]; 8],
    pub repos_len: u8,
    pub jurisdictions: [[u8; 16]; 8],
    pub jurisdictions_len: u8,
    pub actions: [[u8; 32]; 16],
    pub actions_len: u8,
    pub checksum: [u8; 64],
}

impl IDEISProjectManifest {
    pub fn iter_actions<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        ManifestStrIter {
            buf: &self.actions,
            len: self.actions_len,
            idx: 0,
        }
    }
}

struct ManifestStrIter<'a> {
    buf: &'a [[u8; 32]; 16],
    len: u8,
    idx: u8,
}

impl<'a> Iterator for ManifestStrIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let raw = &self.buf[self.idx as usize];
        self.idx += 1;
        let n = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
        core::str::from_utf8(&raw[..n]).ok()
    }
}
