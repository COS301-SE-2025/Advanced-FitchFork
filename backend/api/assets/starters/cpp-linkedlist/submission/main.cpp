#include <iostream>
#include "LinkedList.hpp"

int main()
{
    LinkedList lst;
    lst.push_back(1);
    lst.push_back(2);
    lst.push_back(3);

    int x;
    lst.pop_front(x); // tail bug may manifest when popping to empty later

    lst.insert(1, 99);
    lst.erase(2); // tail bug if this was last

    auto v = lst.toVector();
    std::cout << "List:";
    for (int n : v)
        std::cout << " " << n;
    std::cout << "\nsize=" << lst.size() << "\n";
    return 0;
}
