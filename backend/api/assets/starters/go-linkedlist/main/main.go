package main

import (
    "fmt"
    "os"
)

const DELIM = "&-=-&"

func section(name string) { fmt.Printf("%s %s\n", DELIM, name) }

func printList(lst *LinkedList, label string) {
    if label != "" { fmt.Printf("%s: ", label) }
    vs := lst.ToSlice()
    fmt.Printf("[")
    for i, v := range vs {
        if i > 0 { fmt.Printf(" ") }
        fmt.Printf("%d", v)
    }
    fmt.Printf("] size=%d\n", lst.Len())
}

func task1_basic_ops() {
    section("start-task1")

    lst := New()
    section("empty-list")
    fmt.Printf("empty=%t size=%d\n", lst.IsEmpty(), lst.Len())

    section("push_front_back")
    lst.PushFront(2)
    lst.PushBack(5)
    lst.PushFront(1)
    printList(lst, "after-push")

    section("front_back")
    f, _ := lst.Front()
    b, _ := lst.Back()
    fmt.Printf("front=%d back=%d\n", f, b)

    section("pop_front")
    ok, x := lst.PopFront()
    fmt.Printf("ok=%t popped=%d\n", ok, x)
    printList(lst, "after-pop")

    section("clear")
    lst.Clear()
    fmt.Printf("empty=%t size=%d\n", lst.IsEmpty(), lst.Len())

    section("pop_last_then_push")
    one := New()
    one.PushBack(7)
    ok2, y := one.PopFront()
    fmt.Printf("ok=%t popped=%d\n", ok2, y)
    fmt.Printf("empty=%t size=%d\n", one.IsEmpty(), one.Len())
    one.PushBack(99)
    printList(one, "after-pop-last-then-push")
}

func task2_insert_erase() {
    section("start-task2")
    lst := New()
    for i := 1; i <= 5; i++ { lst.PushBack(i) }
    printList(lst, "seed")

    section("insert")
    fmt.Printf("ok=%t\n", lst.InsertAt(0, 100))
    fmt.Printf("ok=%t\n", lst.InsertAt(3, 200))
    fmt.Printf("ok=%t\n", lst.InsertAt(lst.Len(), 300))
    printList(lst, "after-insert")

    section("erase")
    fmt.Printf("ok=%t\n", lst.RemoveAt(0))
    fmt.Printf("ok=%t\n", lst.RemoveAt(2))
    fmt.Printf("ok=%t\n", lst.RemoveAt(lst.Len()-1))
    printList(lst, "after-erase")

    section("erase-tail-then-push")
    okTail := lst.RemoveAt(lst.Len()-1)
    fmt.Printf("ok=%t\n", okTail)
    lst.PushBack(999)
    printList(lst, "after-erase-tail-then-push")
}

func task3_copy_move() {
    section("start-task3")
    a := New()
    for i := 0; i < 4; i++ { a.PushBack(i*10) }
    printList(a, "a")

    section("copy-ctor")
    b := a.Copy()
    printList(b, "b")

    section("modify-original")
    a.PushBack(40)
    _ = a.RemoveAt(1)
    printList(a, "a-after")
    printList(b, "b-unchanged")

    section("steal/move-sim")
    c := MoveFrom(a)
    printList(c, "c")
    printList(a, "a-moved-from")

    section("move-assign-sim")
    d := New()
    d.MoveAssignFrom(c)
    printList(d, "d")
    printList(c, "c-moved-from")
}

func main() {
    which := ""
    if len(os.Args) >= 2 { which = os.Args[1] }
    switch which {
    case "task1": task1_basic_ops()
    case "task2": task2_insert_erase()
    case "task3": task3_copy_move()
    default:
        task1_basic_ops(); task2_insert_erase(); task3_copy_move()
    }
}

