#pragma once
#include <stddef.h>

typedef struct Node {
    int value;
    struct Node* next;
} Node;

typedef struct LinkedList {
    Node* head;
    Node* tail;
    size_t size;
} LinkedList;

void ll_init(LinkedList* l);
void ll_clear(LinkedList* l);
int  ll_empty(const LinkedList* l);
size_t ll_size(const LinkedList* l);

void ll_push_front(LinkedList* l, int v);
void ll_push_back(LinkedList* l, int v);
int  ll_pop_front(LinkedList* l, int* out);

int ll_front(const LinkedList* l, int* out);
int ll_back(const LinkedList* l, int* out);

int ll_insert(LinkedList* l, size_t index, int value);
int ll_erase(LinkedList* l, size_t index);

LinkedList ll_copy(const LinkedList* src);
LinkedList ll_move_from(LinkedList* src);
void ll_move_assign_from(LinkedList* dst, LinkedList* src);

