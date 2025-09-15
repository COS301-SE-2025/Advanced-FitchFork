from __future__ import annotations
from typing import Optional, List


class _Node:
    __slots__ = ("value", "next")
    def __init__(self, v: int):
        self.value: int = v
        self.next: Optional["_Node"] = None


class LinkedList:
    def __init__(self, other: Optional["LinkedList"] = None):
        self._head: Optional[_Node] = None
        self._tail: Optional[_Node] = None
        self._size: int = 0
        if other is not None:
            self.copy_from(other)

    # ---- basics ----
    def empty(self) -> bool:
        return self._size == 0

    def size(self) -> int:
        return self._size

    def clear(self) -> None:
        n = self._head
        while n is not None:
            nxt = n.next
            n.next = None
            n = nxt
        self._head = self._tail = None
        self._size = 0

    # ---- push/pop ----
    def push_front(self, v: int) -> None:
        n = _Node(v)
        n.next = self._head
        self._head = n
        if self._tail is None:
            self._tail = n
        self._size += 1

    def push_back(self, v: int) -> None:
        n = _Node(v)
        if self._tail is None:
            self._head = self._tail = n
        else:
            self._tail.next = n
            self._tail = n
        self._size += 1

    def pop_front(self) -> (bool, Optional[int]):
        if self._head is None:
            return False, None
        n = self._head
        self._head = n.next
        if self._head is None:
            self._tail = None
        self._size -= 1
        return True, n.value

    # ---- access ----
    def front(self) -> int:
        if self._head is None:
            raise IndexError("front from empty list")
        return self._head.value

    def back(self) -> int:
        if self._tail is None:
            raise IndexError("back from empty list")
        return self._tail.value

    # ---- indexed ops ----
    def insert(self, index: int, value: int) -> bool:
        if index < 0 or index > self._size:
            return False
        if index == 0:
            self.push_front(value)
            return True
        if index == self._size:
            self.push_back(value)
            return True
        prev = self._head
        for _ in range(index - 1):
            assert prev is not None
            prev = prev.next
        n = _Node(value)
        n.next = prev.next
        prev.next = n
        self._size += 1
        return True

    def erase(self, index: int) -> bool:
        if index < 0 or index >= self._size:
            return False
        if index == 0:
            ok, _ = self.pop_front()
            return ok
        prev = self._head
        for _ in range(index - 1):
            assert prev is not None
            prev = prev.next
        victim = prev.next
        assert victim is not None
        prev.next = victim.next
        if victim is self._tail:
            self._tail = prev
        self._size -= 1
        return True

    # ---- utils ----
    def to_list(self) -> List[int]:
        out: List[int] = []
        n = self._head
        while n is not None:
            out.append(n.value)
            n = n.next
        return out

    def copy(self) -> "LinkedList":
        return LinkedList(self)

    def copy_from(self, other: "LinkedList") -> None:
        self.clear()
        n = other._head
        while n is not None:
            self.push_back(n.value)
            n = n.next

    def steal_from(self, other: "LinkedList") -> None:
        """Move/steal nodes from `other` into `self`."""
        self.clear()
        self._head, self._tail, self._size = other._head, other._tail, other._size
        other._head = other._tail = None
        other._size = 0
