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
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push_front(&mut self, elem: T) {
        let new = Box::new(Node { elem, next: self.head.take() });
        self.head = Some(new);
        self.len += 1;
    }

    pub fn push_back(&mut self, elem: T) {
        // Cursor with explicit match avoids overlapping mutable borrows.
        let mut cur = &mut self.head;
        loop {
            match cur {
                Some(ref mut node) => {
                    cur = &mut node.next;
                }
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

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|n| &n.elem)
    }

    pub fn back(&self) -> Option<&T> {
        let mut cur = self.head.as_ref();
        while let Some(node) = cur {
            if node.next.is_none() { return Some(&node.elem); }
            cur = node.next.as_ref();
        }
        None
    }

    pub fn insert_at(&mut self, idx: usize, elem: T) -> Result<(), &'static str> {
        if idx > self.len { return Err("index out of bounds"); }
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
    pub fn move_from(src: &mut Self) -> Self {
        let head = src.head.take();
        let len = src.len;
        src.len = 0;
        Self { head, len }
    }

    pub fn move_assign_from(&mut self, src: &mut Self) {
        self.clear();
        self.head = src.head.take();
        self.len = std::mem::take(&mut src.len);
    }
}
