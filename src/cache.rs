use arrayvec::{Array, ArrayVec};
use std::fmt;

pub struct Entry<T> {
    value: T,
    prev: usize,
    next: usize,
}

pub struct LRUCache<A: Array> {
    entries: ArrayVec<A>,
    head: usize,
    tail: usize,
    length: usize,
}

impl<A: Array> Default for LRUCache<A> {
    fn default() -> Self {
        LRUCache {
            entries: ArrayVec::new(),
            head: 0,
            tail: 0,
            length: 0,
        }
    }
}

struct IterMut<'a, A: 'a + Array> {
    cache: &'a mut LRUCache<A>,
    index: usize,
    done: bool,
}

impl<'a, T, A> Iterator for IterMut<'a, A>
where
T: 'a,
A: 'a + Array<Item = Entry<T>>,
{
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let entry = unsafe { &mut *(&mut self.cache.entries[self.index] as *mut Entry<T>) };
        let index = self.index;

        if self.index == self.cache.tail {
            self.done = true;
        }

        self.index = entry.next;

        Some((index, &mut entry.value))
    }
}

impl<T, A> LRUCache<A>
where
A: Array<Item = Entry<T>>,
{
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn clear_elements(&mut self) {
        self.entries.clear();
        self.head = 0;
        self.tail = 0;
        self.length = self.entries.len();
    }

    pub fn get_front(&self) -> Option<&T> {
        self.entries.get(self.head).map(|e| &e.value)
    }

    pub fn get_front_mut(&mut self) -> Option<&mut T> {
        self.entries.get_mut(self.head).map(|e| &mut e.value)
    }

    pub fn call<F>(&mut self, mut condition: F) -> bool
    where
    F: FnMut(&T) -> bool,
    {
        match self.iter_mut().find(|&(_, ref x)| condition(x)) {
            Some((i, _)) => {
                self.call_on_index(i);
                true
            }
            None => false,
        }
    }

    pub fn lookup<F, R>(&mut self, mut condition: F) -> Option<R>
    where
    F: FnMut(&mut T) -> Option<R>,
    {
        let mut result = None;

        for (i, entry) in self.iter_mut() {
            if let Some(r) = condition(entry) {
                result = Some((i, r));
                break;
            }
        }

        match result {
            None => None,
            Some((i, r)) => {
                self.call_on_index(i);
                Some(r)
            }
        }
    }

    fn call_on_index(&mut self, index: usize) {
        if index != self.head {
            self.remove(index);
            self.length += 1;

            if self.entries.len() == 1 {
                self.tail = index;
            } else {
                self.entries[index].next = self.head;
                self.entries[self.head].prev = index;
            }

            self.head = index;
        }
    }

    fn insert(&mut self, value: T) {
        let entry = Entry {
            value,
            prev: 0,
            next: 0,
        };

        let new_head = if self.length == self.entries.capacity() {
            let last_index = {
                let old_tail = self.tail;
                let new_tail = self.entries[old_tail].prev;
                self.tail = new_tail;
                old_tail as usize
            };
            self.entries[last_index] = entry;
            last_index
        } else {
            self.entries.push(entry);
            self.length += 1;
            self.entries.len() - 1
        };

        if self.entries.len() == 1 {
            self.tail = new_head;
        } else {
            self.entries[new_head].next = self.head;
            self.entries[self.head].prev = new_head;
        }

        self.head = new_head;
    }

    fn remove(&mut self, index: usize) {
        assert!(!self.is_empty());

        let prev = self.entries[index].prev;
        let next = self.entries[index].next;

        if index == self.head {
            self.head = next;
        } else {
            self.entries[prev].next = next;
        }

        if index == self.tail {
            self.tail = prev;
        } else {
            self.entries[next].prev = prev;
        }

        self.length -= 1;
    }

    fn iter_mut(&mut self) -> IterMut<A> {
        IterMut {
            index: self.head,
            done: self.is_empty(),
            cache: self,
        }
    }
}

// No implementable dado el tipo <T>
impl<A: Array> fmt::Display for LRUCache<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.entries.len(),)
    }
}

// Testing
#[cfg(test)]
mod tests {

    use super::{LRUCache, Entry, Array};

    type StringTest = LRUCache<[Entry<String>; 3]>;

    fn items<T, A>(cache: &mut LRUCache<A>) -> Vec<T>
    where
    T: Clone,
    A: Array<Item = Entry<T>>,
    {
        cache.iter_mut().map(|(_, x)| x.clone()).collect()
    }

#[test]
fn test_empty() {
        let mut elements = StringTest::default();
        assert_eq!(elements.len(), 0);
        assert_eq!(items(&mut elements), Vec::<String>::new());
    }

#[test]
fn test_insert() {
        let mut elements = StringTest::default();

        elements.insert("David".to_string());
        elements.insert("Tomas".to_string());
        elements.insert("Danilo".to_string());
        assert_eq!(items(&mut elements), ["Danilo", "Tomas", "David"]);

        elements.insert("Sebastian".to_string());
        assert_eq!(elements.len(), 3);
        assert_eq!(items(&mut elements), ["Sebastian", "Danilo", "Tomas"]);

        elements.insert("Daniel".to_string());
        elements.insert("Patata".to_string());
        elements.insert("Isabel".to_string());

        assert_eq!(elements.len(), 3);
        assert_eq!(items(&mut elements), ["Isabel", "Patata", "Daniel"]);
    }

#[test]
fn test_remove() {
        let mut elements = StringTest::default();

        elements.insert("David".to_string());
        elements.insert("Tomas".to_string());
        elements.insert("Danilo".to_string());
        assert_eq!(items(&mut elements), ["Danilo", "Tomas", "David"]);

        elements.remove(0);
        assert_eq!(items(&mut elements), ["Danilo", "Tomas"]);
    }

#[test]
fn test_lookup() {
        let mut elements = StringTest::default();

        elements.insert("David".to_string());
        elements.insert("Tomas".to_string());
        elements.insert("Danilo".to_string());

        let result = elements.lookup(|x| if *x == "Sebastian" { Some(()) } else { None });
        assert_eq!(result, None);
        assert_eq!(items(&mut elements), ["Danilo", "Tomas", "David"]);

        let result = elements.lookup(|x| if *x == "David" { Some("David") } else { None });
        assert_eq!(result, Some("David"));
        assert_eq!(items(&mut elements), ["David", "Danilo", "Tomas"]);
    }

#[test]
fn test_get_front() {
        let mut elements = StringTest::default();
        assert_eq!(elements.get_front(), None);

        elements.insert("David".to_string());
        elements.insert("Tomas".to_string());
        assert_eq!(elements.get_front(), Some(&"Tomas".to_string()));

        elements.call(|x| *x == "David");
        assert_eq!(elements.get_front(), Some(&"David".to_string()));
    }

#[test]
fn test_clear() {
        let mut elements = StringTest::default();

        elements.insert("David".to_string());
        elements.clear_elements();
        assert_eq!(elements.len(), 0);
        assert_eq!(items(&mut elements), Vec::<String>::new());

        elements.insert("David".to_string());
        elements.insert("Tomas".to_string());
        elements.insert("Danilo".to_string());
        assert_eq!(items(&mut elements), ["Danilo", "Tomas", "David"]);

        elements.clear_elements();
        assert_eq!(items(&mut elements), Vec::<String>::new());
    }

#[test]
fn test_call() {
        let mut elements = StringTest::default();

        elements.insert("David".to_string());
        elements.insert("Tomas".to_string());
        elements.insert("Danilo".to_string());
        assert_eq!(items(&mut elements), ["Danilo", "Tomas", "David"]);

        elements.call(|x| *x == "Sebastian");
        assert_eq!(items(&mut elements), ["Danilo", "Tomas", "David"]);

        elements.call(|x| *x == "Tomas");
        assert_eq!(items(&mut elements), ["Tomas", "Danilo", "David"]);
    }
}