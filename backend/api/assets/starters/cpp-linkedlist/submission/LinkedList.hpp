#pragma once
#include <cstddef>
#include <vector>

class LinkedList
{
public:
    LinkedList();
    LinkedList(const LinkedList &other);
    LinkedList(LinkedList &&other) noexcept;
    LinkedList &operator=(const LinkedList &other);
    LinkedList &operator=(LinkedList &&other) noexcept;
    ~LinkedList();

    bool empty() const noexcept;
    std::size_t size() const noexcept;

    void clear() noexcept;

    void push_front(int v);
    void push_back(int v);
    bool pop_front(int &out);

    int &front();
    const int &front() const;
    int &back();
    const int &back() const;

    bool insert(std::size_t index, int value);
    bool erase(std::size_t index);

    std::vector<int> toVector() const;

private:
    struct Node
    {
        int value;
        Node *next;
        explicit Node(int v) : value(v), next(nullptr) {}
    };
    Node *head_;
    Node *tail_;
    std::size_t size_;

    void copyFrom(const LinkedList &other);
};
