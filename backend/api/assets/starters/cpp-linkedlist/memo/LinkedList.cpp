#include "LinkedList.hpp"

LinkedList::LinkedList() : head_(nullptr), tail_(nullptr), size_(0) {}
LinkedList::LinkedList(const LinkedList &other) : LinkedList() { copyFrom(other); }

LinkedList::LinkedList(LinkedList &&other) noexcept
    : head_(other.head_), tail_(other.tail_), size_(other.size_)
{
    other.head_ = other.tail_ = nullptr;
    other.size_ = 0;
}

LinkedList &LinkedList::operator=(const LinkedList &other)
{
    if (this != &other)
    {
        clear();
        copyFrom(other);
    }
    return *this;
}
LinkedList &LinkedList::operator=(LinkedList &&other) noexcept
{
    if (this != &other)
    {
        clear();
        head_ = other.head_;
        tail_ = other.tail_;
        size_ = other.size_;
        other.head_ = other.tail_ = nullptr;
        other.size_ = 0;
    }
    return *this;
}
LinkedList::~LinkedList() { clear(); }

bool LinkedList::empty() const noexcept { return size_ == 0; }
std::size_t LinkedList::size() const noexcept { return size_; }

void LinkedList::clear() noexcept
{
    for (auto *n = head_; n;)
    {
        auto *next = n->next;
        delete n;
        n = next;
    }
    head_ = tail_ = nullptr;
    size_ = 0;
}

void LinkedList::push_front(int v)
{
    auto *n = new Node(v);
    n->next = head_;
    head_ = n;
    if (!tail_)
        tail_ = n;
    ++size_;
}
void LinkedList::push_back(int v)
{
    auto *n = new Node(v);
    if (tail_)
    {
        tail_->next = n;
        tail_ = n;
    }
    else
    {
        head_ = tail_ = n;
    }
    ++size_;
}
bool LinkedList::pop_front(int &out)
{
    if (!head_)
        return false;
    auto *n = head_;
    out = n->value;
    head_ = n->next;
    if (!head_)
        tail_ = nullptr;
    delete n;
    --size_;
    return true;
}

int &LinkedList::front() { return head_->value; }
const int &LinkedList::front() const { return head_->value; }
int &LinkedList::back() { return tail_->value; }
const int &LinkedList::back() const { return tail_->value; }

bool LinkedList::insert(std::size_t index, int value)
{
    if (index > size_)
        return false;
    if (index == 0)
    {
        push_front(value);
        return true;
    }
    if (index == size_)
    {
        push_back(value);
        return true;
    }
    auto *prev = head_;
    for (std::size_t i = 0; i + 1 < index; ++i)
        prev = prev->next;
    auto *n = new Node(value);
    n->next = prev->next;
    prev->next = n;
    ++size_;
    return true;
}
bool LinkedList::erase(std::size_t index)
{
    if (index >= size_)
        return false;
    if (index == 0)
    {
        int tmp;
        return pop_front(tmp);
    }
    auto *prev = head_;
    for (std::size_t i = 0; i + 1 < index; ++i)
        prev = prev->next;
    auto *victim = prev->next;
    prev->next = victim->next;
    if (victim == tail_)
        tail_ = prev;
    delete victim;
    --size_;
    return true;
}

std::vector<int> LinkedList::toVector() const
{
    std::vector<int> out;
    out.reserve(size_);
    for (auto *n = head_; n; n = n->next)
        out.push_back(n->value);
    return out;
}
void LinkedList::copyFrom(const LinkedList &other)
{
    for (auto *n = other.head_; n; n = n->next)
        push_back(n->value);
}
