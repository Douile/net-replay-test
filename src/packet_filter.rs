//! Filtering primitives for packet data

use std::collections::HashMap;

use crate::{packet::PacketDirection, QueryReplay};

#[derive(Clone, Debug)]
pub enum FilterError {
    /// Tried to replace an empty buffer
    EmptyReplace,
    /// Tried to replace with a different length replacement
    MismatchReplaceLen,
    /// Expected to replace buffer but it was not found
    ReplacementNotFound,
}

struct ReplacePtr {
    start_pos: usize,
    len: usize,
}

impl ReplacePtr {
    const fn new(start_pos: usize) -> Self {
        Self { start_pos, len: 1 }
    }
}

// TODO: Test if replacing all replacements in one sweep is more efficient
/// Replace raw bytes with other raw bytes
pub fn raw_replace(
    buffer: &mut [u8],
    to_replace: &[u8],
    replacement: &[u8],
) -> Result<usize, FilterError> {
    if to_replace.len() == 0 {
        return Err(FilterError::EmptyReplace);
    }
    if to_replace.len() != replacement.len() {
        return Err(FilterError::MismatchReplaceLen);
    }

    let mut state: Vec<ReplacePtr> = Vec::with_capacity(buffer.len());
    let mut matches: Vec<ReplacePtr> = Vec::with_capacity(buffer.len());

    // Find matches
    for (i, byte) in buffer.iter().enumerate() {
        for j in (0..state.len()).rev() {
            if to_replace[state[j].len].eq(byte) {
                state[j].len += 1;
                if state[j].len >= to_replace.len() {
                    matches.push(state.swap_remove(j));
                }
            } else {
                state.swap_remove(j);
            }
        }

        if to_replace[0].eq(byte) {
            state.push(ReplacePtr::new(i));
        }
    }

    for replace_match in &matches {
        let section =
            &mut buffer[replace_match.start_pos..replace_match.start_pos + replace_match.len];
        section.copy_from_slice(replacement);
    }

    Ok(matches.len())
}

/// Replace occurrences of to_replace in buffer with replacement
#[inline]
pub fn string_replace(
    buffer: &mut [u8],
    to_replace: &str,
    replacement: &str,
) -> Result<usize, FilterError> {
    raw_replace(buffer, to_replace.as_bytes(), replacement.as_bytes())
}

struct InfiniteSequence<'a, T> {
    source: &'a [T],
    pointers: Vec<usize>,
    pos: usize,
}

impl<'a, T: Clone> InfiniteSequence<'a, T> {
    pub fn new(source: &'a [T]) -> InfiniteSequence<'a, T> {
        assert!(!source.is_empty());
        InfiniteSequence {
            source,
            pointers: vec![0],
            pos: 0,
        }
    }
}

impl<'a, T: Clone> Iterator for InfiniteSequence<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pos) = self.pos.checked_sub(1) {
            self.pos = pos;
        } else {
            self.pos = self.pointers.len() - 1;
        }
        if self.pointers[self.pos] >= self.source.len() {
            if self.pos == self.pointers.len() - 1 {
                self.pointers.push(0);
            }
            self.pointers[self.pos] = 0;
        }
        let r = self.source[self.pointers[self.pos]].clone();
        if self.pos == 0 {
            let mut i = 0;
            loop {
                if i >= self.pointers.len() {
                    self.pointers.push(0);
                    break;
                }

                self.pointers[i] += 1;
                if self.pointers[i] >= self.source.len() {
                    self.pointers[i] = 0;
                } else {
                    break;
                }

                i += 1;
            }
        }
        Some(r)
    }
}

/// Replace all names in a query replay with censored version
pub fn packet_name_replace(query_replay: &mut QueryReplay) -> Result<(), FilterError> {
    let source = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();
    let mut generator_ = InfiniteSequence::new(source);
    let generator = &mut generator_;

    let mut name_replacements = HashMap::with_capacity(query_replay.value.player_names.len());
    for name in &query_replay.value.player_names {
        name_replacements.insert(name, generator.take(name.len()).collect::<Vec<_>>());
    }

    for packet in query_replay
        .packets
        .iter_mut()
        .filter(|packet| packet.direction == PacketDirection::FromServer)
    {
        for (name, replacement) in name_replacements.iter() {
            raw_replace(&mut packet.data, name.as_bytes(), replacement)?;
        }
    }

    query_replay.value.player_names = name_replacements
        .into_values()
        .map(|name| String::from_utf8_lossy(&name).into_owned())
        .collect();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::string_replace;
    use super::InfiniteSequence;

    #[test]
    fn replace_string() {
        let mut buffer = String::from("foo: This is a foo test");
        let r = string_replace(unsafe { buffer.as_bytes_mut() }, "foo", "bar").unwrap();
        assert_eq!(r, 2);
        assert_eq!(buffer, "bar: This is a bar test");
    }

    #[test]
    fn test_infinite() {
        let source = [0, 1, 2];
        let inf = InfiniteSequence::new(&source);

        let generated: Vec<u8> = inf.take(12).collect();
        assert_eq!(generated, vec![0, 1, 2, 0, 0, 0, 1, 0, 2, 1, 0, 1]);
    }
}
