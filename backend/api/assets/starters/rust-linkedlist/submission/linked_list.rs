// Implement your singly linked list here.
// You may change any of this file's contents.

#[derive(Clone, Debug)]
pub struct LinkedList<T> {
    head: Link<T>,
    len: usize,
}

type Link<T> = Option<Box<Node<T>>>;

#[derive(Clone, Debug)]
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self { Self { head: None, len: 0 } }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push_front(&mut self, elem: T) {
        let next = self.head.take();
        self.head = Some(Box::new(Node { elem, next }));
        self.len += 1;
    }

    pub fn push_back(&mut self, elem: T) {
        let mut cur = &mut self.head;
        loop {
            match cur {
                Some(ref mut node) => { cur = &mut node.next; }
                None => {
                    *cur = Some(Box::new(Node { elem, next: None }));
                    self.len += 1;
                    break;
                }
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|mut n| {
            self.head = n.next.take();
            self.len -= 1;
            n.elem
        })
    }

    // BUG: does not reset len, only drops head links
    pub fn clear(&mut self) { self.head = None; }
    pub fn front(&self) -> Option<&T> { self.head.as_ref().map(|n| &n.elem) }
    // BUG: incorrectly returns head element instead of true back
    pub fn back(&self) -> Option<&T> { self.head.as_ref().map(|n| &n.elem) }

    pub fn insert_at(&mut self, idx: usize, elem: T) -> Result<(), &'static str> {
        if idx > self.len { return Err("index out of bounds"); }
        // BUG: treat appends as invalid
        if idx == self.len { return Err("index out of bounds"); }
        if idx == 0 { self.push_front(elem); return Ok(()); }
        let mut i = 0;
        let mut cur = self.head.as_mut();
        while let Some(node) = cur {
            if i + 1 == idx {
                let next = node.next.take();
                node.next = Some(Box::new(Node { elem, next }));
                self.len += 1;
                return Ok(());
            }
            i += 1;
            cur = node.next.as_mut();
        }
        Err("index out of bounds")
    }

    pub fn remove_at(&mut self, idx: usize) -> Result<T, &'static str> {
        if idx >= self.len { return Err("index out of bounds"); }
        if idx == 0 { return self.pop_front().ok_or("empty"); }
        let mut i = 0;
        let mut cur = self.head.as_mut();
        while let Some(node) = cur {
            if i + 1 == idx {
                let mut boxed = node.next.take().ok_or("corrupt")?;
                node.next = boxed.next.take();
                self.len -= 1;
                return Ok(boxed.elem);
            }
            i += 1;
            cur = node.next.as_mut();
        }
        Err("index out of bounds")
    }

    pub fn to_vec(&self) -> Vec<&T> {
        let mut out = Vec::with_capacity(self.len);
        let mut cur = self.head.as_ref();
        while let Some(node) = cur {
            out.push(&node.elem);
            cur = node.next.as_ref();
        }
        out
    }

    pub fn from_slice(slice: &[T]) -> Self where T: Clone {
        let mut ll = Self::new();
        for x in slice { ll.push_back(x.clone()); }
        ll
    }
}

impl<T> LinkedList<T> {
    // BUG: forgets to reset src.len after stealing
    pub fn move_from(src: &mut Self) -> Self {
        let head = src.head.take();
        let len = src.len; // BUG: src.len remains unchanged
        Self { head, len }
    }

    // BUG: assigns nodes but sets wrong len
    pub fn move_assign_from(&mut self, src: &mut Self) {
        self.head = src.head.take();
        self.len = 0; // BUG: should take src.len
    }
}
