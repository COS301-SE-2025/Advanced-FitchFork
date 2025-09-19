package main

// Spec skeleton (students implement these methods)

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

func (l *LinkedList) Clear() { panic("TODO: Clear") }
func (l *LinkedList) PushFront(v int) { panic("TODO: PushFront") }
func (l *LinkedList) PushBack(v int) { panic("TODO: PushBack") }
func (l *LinkedList) PopFront() (bool, int) { panic("TODO: PopFront") }
func (l *LinkedList) Front() (int, bool) { panic("TODO: Front") }
func (l *LinkedList) Back() (int, bool) { panic("TODO: Back") }
func (l *LinkedList) InsertAt(idx int, v int) bool { panic("TODO: InsertAt") }
func (l *LinkedList) RemoveAt(idx int) bool { panic("TODO: RemoveAt") }
func (l *LinkedList) ToSlice() []int { panic("TODO: ToSlice") }

func (l *LinkedList) Copy() *LinkedList { panic("TODO: Copy") }
func MoveFrom(src *LinkedList) *LinkedList { panic("TODO: MoveFrom") }
func (l *LinkedList) MoveAssignFrom(src *LinkedList) { panic("TODO: MoveAssignFrom") }

