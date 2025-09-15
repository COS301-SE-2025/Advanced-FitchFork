#include <iostream>
#include <string>
#include <vector>
#include <cstdlib>
#include "LinkedList.hpp" // resolved via -I. -Imemo -Ispec

// Delimiter token must match ExecutionConfig.default_deliminator() => "&-=-&"
static constexpr const char *DELIM = "&-=-&";

static void print_section(const std::string &name)
{
    std::cout << DELIM << " " << name << "\n";
}

static void print_list(const LinkedList &lst, const std::string &label = "")
{
    if (!label.empty())
        std::cout << label << ": ";
    std::cout << "[";
    bool first = true;
    for (int v : lst.toVector())
    {
        if (!first)
            std::cout << " ";
        std::cout << v;
        first = false;
    }
    std::cout << "] size=" << lst.size() << "\n";
}

// ───────────────────────── tasks ─────────────────────────

static void task1_basic_ops()
{
    print_section("start-task1");

    LinkedList lst;
    print_section("empty-list");
    std::cout << std::boolalpha << "empty=" << lst.empty() << " size=" << lst.size() << "\n";

    print_section("push_front_back");
    lst.push_front(2);
    lst.push_back(5);
    lst.push_front(1);
    print_list(lst, "after-push");

    print_section("front_back");
    std::cout << "front=" << lst.front() << " back=" << lst.back() << "\n";

    print_section("pop_front");
    int out;
    bool ok = lst.pop_front(out);
    std::cout << "ok=" << ok << " popped=" << (ok ? std::to_string(out) : std::string("N/A")) << "\n";
    print_list(lst, "after-pop");

    print_section("clear");
    lst.clear();
    std::cout << "empty=" << lst.empty() << " size=" << lst.size() << "\n";
}

static void task2_insert_erase()
{
    print_section("start-task2");

    LinkedList lst;
    for (int i = 1; i <= 5; ++i)
        lst.push_back(i); // [1 2 3 4 5]
    print_list(lst, "seed");

    print_section("insert");
    bool iok = lst.insert(0, 100); // [100 1 2 3 4 5]
    std::cout << "ok=" << iok << "\n";
    iok = lst.insert(3, 200); // [100 1 2 200 3 4 5]
    std::cout << "ok=" << iok << "\n";
    iok = lst.insert(lst.size(), 300); // append -> [... 5 300]
    std::cout << "ok=" << iok << "\n";
    print_list(lst, "after-insert");

    print_section("erase");
    bool eok = lst.erase(0); // remove 100
    std::cout << "ok=" << eok << "\n";
    eok = lst.erase(2); // remove 200 (now at index 2)
    std::cout << "ok=" << eok << "\n";
    eok = lst.erase(lst.size() - 1); // remove 300 (tail)
    std::cout << "ok=" << eok << "\n";
    print_list(lst, "after-erase");
}

static void task3_copy_move()
{
    print_section("start-task3");

    LinkedList a;
    for (int i = 0; i < 4; ++i)
        a.push_back(i * 10); // [0 10 20 30]
    print_list(a, "a");

    print_section("copy-ctor");
    LinkedList b = a;
    print_list(b, "b");

    print_section("modify-original");
    a.push_back(40);
    a.erase(1);
    print_list(a, "a-after");
    print_list(b, "b-unchanged");

    print_section("move-ctor");
    LinkedList c = std::move(a);
    print_list(c, "c");
    print_list(a, "a-moved-from");

    print_section("move-assign");
    LinkedList d;
    d.push_back(7);
    d = std::move(c);
    print_list(d, "d");
    print_list(c, "c-moved-from");
}

// ───────────────────────── entry ─────────────────────────

int main(int argc, char **argv)
{
    if (argc >= 2)
    {
        std::string which = argv[1];
        if (which == "task1")
        {
            task1_basic_ops();
            return 0;
        }
        if (which == "task2")
        {
            task2_insert_erase();
            return 0;
        }
        if (which == "task3")
        {
            task3_copy_move();
            return 0;
        }
        std::cerr << "unknown task: " << which << "\n";
        return 2;
    }
    // default: run all tasks in order
    task1_basic_ops();
    task2_insert_erase();
    task3_copy_move();
    return 0;
}
