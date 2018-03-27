use std::ffi::{OsStr, OsString};
use std::cmp::Ordering;

struct CursorMut<'a> {
    tree: &'a mut Tree,
    stack: Vec<usize>
}

struct Tree {
    entries: Vec<Entry>
}

enum Entry {
    Dir {
        name: OsString,
        symlink: bool,
        ignored: bool,
        span: usize,
        last_entry_span: usize,
    },
    File {
        name: OsString,
        symlink: bool,
        ignored: bool,
    }
}

impl Entry {
    fn name(&self) -> &OsStr {
        match self {
            &Entry::Dir { ref name, .. } => name,
            &Entry::File { ref name, .. } => name,
        }
    }

    fn span(&self) -> usize {
        match self {
            &Entry::Dir { span, .. } => span,
            &Entry::File { .. } => 1,
        }
    }
}

impl<'a> CursorMut<'a> {
    pub fn add_entry(&mut self, entry: Entry) -> Result<(), ()> {
        let insertion_index;
        let appending;
        let index = *self.stack.last().unwrap();

        match &self.tree.entries[index] {
            &Entry::File { .. } => return Err(()),
            &Entry::Dir { span, last_entry_span, .. } => {
                let last_child_index = index + span - last_entry_span;
                if entry.name() > self.tree.entries[last_child_index].name() {
                    appending = true;
                    insertion_index = index + span;
                } else {
                    appending = false;
                    let mut sibling_index = index + 1;
                    loop {
                        let sibling = &self.tree.entries[sibling_index];
                        match entry.name().cmp(sibling.name()) {
                            Ordering::Less => sibling_index += sibling.span(),
                            Ordering::Equal => return Err(()),
                            Ordering::Greater => {
                                insertion_index = sibling_index + sibling.span();
                                break;
                            }
                        }
                    }
                }
            }
        }
        self.tree.entries.insert(insertion_index, entry);

        match &mut self.tree.entries[index] {
            &mut Entry::Dir { ref mut span, ref mut last_entry_span, .. } => {
                *span += 1;
                if appending {
                    *last_entry_span += 1;
                }
            },
            _ => unreachable!()
        }

        for ancestor_index in self.stack.iter_mut().rev().skip(1) {
            match self.tree.entries[*ancestor_index] {
                Entry::Dir { ref mut span, ref mut last_entry_span, .. } => {
                    let last_child_index = *ancestor_index + *span - *last_entry_span;
                    if last_child_index <= index {
                        *last_entry_span += 1;
                    }
                    *span += 1;
                },
                _ => unreachable!()
            }
        }

        Ok(())
    }

    pub fn ascend(&mut self) -> Result<(), ()> {
        if self.stack.len() == 1 {
            Err(())
        } else {
            self.stack.pop();
            Ok(())
        }
    }

    pub fn descend(&mut self) -> Result<(), ()> {
        let index = *self.stack.last().unwrap();
        match &self.tree.entries[index] {
            &Entry::File { .. } => Err(()),
            &Entry::Dir { span, .. } => {
                if span == 0 {
                    Err(())
                } else {
                    self.stack.push(index + 1);
                    Ok(())
                }
            }
        }
    }
}
