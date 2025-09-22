package main

type node struct {
    val  int
    next *node
}

type LinkedList struct {
    head *node
    tail *node
    size int
}

func New() *LinkedList { return &LinkedList{} }
func (l *LinkedList) Len() int { return l.size }
func (l *LinkedList) IsEmpty() bool { return l.size == 0 }

func (l *LinkedList) Clear() {
    for l.head != nil {
        n := l.head
        l.head = n.next
        n.next = nil
    }
    l.tail = nil
    l.size = 0
}

func (l *LinkedList) PushFront(v int) {
    n := &node{val: v, next: l.head}
    l.head = n
    if l.tail == nil { l.tail = n }
    l.size++
}

func (l *LinkedList) PushBack(v int) {
    n := &node{val: v}
    if l.tail == nil { l.head, l.tail = n, n } else { l.tail.next = n; l.tail = n }
    l.size++
}

func (l *LinkedList) PopFront() (bool, int) {
    if l.head == nil { return false, 0 }
    n := l.head
    l.head = n.next
    if l.head == nil { l.tail = nil }
    l.size--
    return true, n.val
}

func (l *LinkedList) Front() (int, bool) {
    if l.head == nil { return 0, false }
    return l.head.val, true
}

func (l *LinkedList) Back() (int, bool) {
    if l.tail == nil { return 0, false }
    return l.tail.val, true
}

func (l *LinkedList) InsertAt(idx int, v int) bool {
    if idx < 0 || idx > l.size { return false }
    if idx == 0 { l.PushFront(v); return true }
    if idx == l.size { l.PushBack(v); return true }
    prev := l.head
    for i := 0; i < idx-1; i++ { prev = prev.next }
    n := &node{val: v, next: prev.next}
    prev.next = n
    l.size++
    return true
}

func (l *LinkedList) RemoveAt(idx int) bool {
    if idx < 0 || idx >= l.size { return false }
    if idx == 0 {
        ok, _ := l.PopFront(); return ok
    }
    prev := l.head
    for i := 0; i < idx-1; i++ { prev = prev.next }
    victim := prev.next
    prev.next = victim.next
    if victim == l.tail { l.tail = prev }
    l.size--
    return true
}

func (l *LinkedList) ToSlice() []int {
    out := make([]int, 0, l.size)
    for n := l.head; n != nil; n = n.next { out = append(out, n.val) }
    return out
}

func (l *LinkedList) Copy() *LinkedList {
    dst := New()
    for n := l.head; n != nil; n = n.next { dst.PushBack(n.val) }
    return dst
}

func MoveFrom(src *LinkedList) *LinkedList {
    dst := New()
    dst.head, dst.tail, dst.size = src.head, src.tail, src.size
    src.head, src.tail, src.size = nil, nil, 0
    return dst
}

func (l *LinkedList) MoveAssignFrom(src *LinkedList) {
    l.Clear()
    l.head, l.tail, l.size = src.head, src.tail, src.size
    src.head, src.tail, src.size = nil, nil, 0
}

