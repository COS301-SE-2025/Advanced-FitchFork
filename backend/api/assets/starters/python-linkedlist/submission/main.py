from LinkedList import LinkedList

def run():
    lst = LinkedList()
    for x in (1, 2, 3):
        lst.push_back(x)
    ok, first = lst.pop_front()

    lst.insert(1, 42)
    lst.erase(2)

    print("list:", lst.to_list())
    print("size:", lst.size())

if __name__ == "__main__":
    run()
