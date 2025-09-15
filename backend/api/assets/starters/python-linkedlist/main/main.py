import os
import sys

# ExecutionConfig.default_deliminator() == "&-=-&"
DELIM = "&-=-&"

# Prefer student's LinkedList.py, then memo/, then spec/
ROOT = os.path.abspath(".")
for p in [ROOT, os.path.join(ROOT, "memo"), os.path.join(ROOT, "spec")]:
    if p not in sys.path:
        sys.path.insert(0, p)

try:
    from LinkedList import LinkedList
except Exception as e:
    print("Failed to import LinkedList:", e)
    sys.exit(2)


def print_section(name: str):
    print(f"{DELIM} {name}")


def print_list(lst: "LinkedList", label: str = ""):
    if label:
        print(f"{label}: ", end="")
    vs = lst.to_list()
    print(f"[{' '.join(map(str, vs))}] size={lst.size()}")


# ───────────────────────── tasks ─────────────────────────

def task1_basic_ops():
    print_section("start-task1")

    lst = LinkedList()
    print_section("empty-list")
    print(f"empty={lst.empty()} size={lst.size()}")

    print_section("push_front_back")
    lst.push_front(2)
    lst.push_back(5)
    lst.push_front(1)
    print_list(lst, "after-push")

    print_section("front_back")
    print(f"front={lst.front()} back={lst.back()}")

    print_section("pop_front")
    ok, out = lst.pop_front()
    print(f"ok={ok} popped={out if ok else 'N/A'}")
    print_list(lst, "after-pop")

    print_section("clear")
    lst.clear()
    print(f"empty={lst.empty()} size={lst.size()}")


def task2_insert_erase():
    print_section("start-task2")

    lst = LinkedList()
    for i in range(1, 6):
        lst.push_back(i)  # [1 2 3 4 5]
    print_list(lst, "seed")

    print_section("insert")
    print("ok=" + str(lst.insert(0, 100)))          # [100 1 2 3 4 5]
    print("ok=" + str(lst.insert(3, 200)))          # [100 1 2 200 3 4 5]
    print("ok=" + str(lst.insert(lst.size(), 300))) # [... 5 300]
    print_list(lst, "after-insert")

    print_section("erase")
    print("ok=" + str(lst.erase(0)))                # remove 100
    print("ok=" + str(lst.erase(2)))                # remove 200 (now at index 2)
    print("ok=" + str(lst.erase(lst.size() - 1)))   # remove 300 (tail)
    print_list(lst, "after-erase")


def task3_copy_aliasing():
    print_section("start-task3")

    a = LinkedList()
    for i in range(4):
        a.push_back(i * 10)  # [0 10 20 30]
    print_list(a, "a")

    print_section("copy-ctor")
    b = a.copy()
    print_list(b, "b")

    print_section("modify-original")
    a.push_back(40)
    a.erase(1)
    print_list(a, "a-after")
    print_list(b, "b-unchanged")

    print_section("steal/move-sim")
    c = LinkedList()
    c.steal_from(a)  # transfer nodes from a → c
    print_list(c, "c")
    print_list(a, "a-moved-from")

    print_section("move-assign-sim")
    d = LinkedList()
    d.steal_from(c)
    print_list(d, "d")
    print_list(c, "c-moved-from")


# ───────────────────────── entry ─────────────────────────

def main():
    which = sys.argv[1] if len(sys.argv) >= 2 else ""
    if which == "task1":
        task1_basic_ops(); return 0
    if which == "task2":
        task2_insert_erase(); return 0
    if which == "task3":
        task3_copy_aliasing(); return 0
    # default: run all
    task1_basic_ops()
    task2_insert_erase()
    task3_copy_aliasing()
    return 0


if __name__ == "__main__":
    sys.exit(main())
