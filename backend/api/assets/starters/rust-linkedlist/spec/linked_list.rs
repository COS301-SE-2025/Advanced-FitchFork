// Spec skeleton for students — do not provide the implementation here.
// Fill in the node structure, fields, and methods as per the assignment.

#[derive(Clone, Debug)]
pub struct LinkedList<T> {
    // TODO: define your storage (e.g., head pointer, length)
    head: Link<T>,
    len: usize,
}

type Link<T> = Option<Box<Node<T>>>;

#[derive(Clone, Debug)]
struct Node<T> {
    // TODO: node fields
    elem: T,
    next: Link<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        // TODO: initialize an empty list
        Self { head: None, len: 0 }
    }

    pub fn len(&self) -> usize {
        // TODO
        self.len
    }

    pub fn is_empty(&self) -> bool {
        // TODO
        self.len == 0
    }

    pub fn push_front(&mut self, _elem: T) {
        // TODO
        unimplemented!("push_front")
    }

    pub fn push_back(&mut self, _elem: T) {
        // TODO
        unimplemented!("push_back")
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // TODO
        unimplemented!("pop_front")
    }

    pub fn clear(&mut self) {
        // TODO
        unimplemented!("clear")
    }

    pub fn front(&self) -> Option<&T> {
        // TODO
        unimplemented!("front")
    }

    pub fn back(&self) -> Option<&T> {
        // TODO
        unimplemented!("back")
    }

    pub fn insert_at(&mut self, _idx: usize, _elem: T) -> Result<(), &'static str> {
        // TODO
        unimplemented!("insert_at")
    }

    pub fn remove_at(&mut self, _idx: usize) -> Result<T, &'static str> {
        // TODO
        unimplemented!("remove_at")
    }

    pub fn to_vec(&self) -> Vec<&T> {
        // TODO: collect references to elements in order
        unimplemented!("to_vec")
    }

    pub fn from_slice(_slice: &[T]) -> Self
    where
        T: Clone,
    {
        // Optional helper
        unimplemented!("from_slice")
    }
}
