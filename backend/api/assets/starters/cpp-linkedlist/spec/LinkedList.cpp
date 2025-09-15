#include "LinkedList.hpp"

// TODO: Provide your implementation so it satisfies the harness in main/main.cpp.

LinkedList::LinkedList() {}
LinkedList::LinkedList(const LinkedList &other) { (void)other; /* TODO */ }
LinkedList::LinkedList(LinkedList &&other) noexcept { (void)other; /* TODO */ }
LinkedList &LinkedList::operator=(const LinkedList &other)
{
    (void)other; /* TODO */
    return *this;
}
LinkedList &LinkedList::operator=(LinkedList &&other) noexcept
{
    (void)other; /* TODO */
    return *this;
}
LinkedList::~LinkedList() {}

bool LinkedList::empty() const noexcept { return true; /* TODO */ }
std::size_t LinkedList::size() const noexcept { return 0; /* TODO */ }

void LinkedList::clear() noexcept { /* TODO */ }

void LinkedList::push_front(int v) { (void)v; /* TODO */ }
void LinkedList::push_back(int v) { (void)v; /* TODO */ }
bool LinkedList::pop_front(int &out)
{
    (void)out;
    return false; /* TODO */
}

int &LinkedList::front() { throw; /* TODO */ }
const int &LinkedList::front() const { throw; /* TODO */ }
int &LinkedList::back() { throw; /* TODO */ }
const int &LinkedList::back() const { throw; /* TODO */ }

bool LinkedList::insert(std::size_t index, int value)
{
    (void)index;
    (void)value;
    return false; /* TODO */
}
bool LinkedList::erase(std::size_t index)
{
    (void)index;
    return false; /* TODO */
}

std::vector<int> LinkedList::toVector() const { return {}; /* TODO */ }
