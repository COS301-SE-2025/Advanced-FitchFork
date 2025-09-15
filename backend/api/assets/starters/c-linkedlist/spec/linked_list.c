#include "linked_list.h"

// Spec skeleton — students implement these.

void ll_init(LinkedList* l){ l->head=0; l->tail=0; l->size=0; }
void ll_clear(LinkedList* l){ /* TODO */ }
int  ll_empty(const LinkedList* l){ return l->size==0; }
size_t ll_size(const LinkedList* l){ return l->size; }

void ll_push_front(LinkedList* l, int v){ (void)l; (void)v; /* TODO */ }
void ll_push_back(LinkedList* l, int v){ (void)l; (void)v; /* TODO */ }
int  ll_pop_front(LinkedList* l, int* out){ (void)l; (void)out; return 0; }

int ll_front(const LinkedList* l, int* out){ (void)l; (void)out; return 0; }
int ll_back(const LinkedList* l, int* out){ (void)l; (void)out; return 0; }

int ll_insert(LinkedList* l, size_t index, int value){ (void)l; (void)index; (void)value; return 0; }
int ll_erase(LinkedList* l, size_t index){ (void)l; (void)index; return 0; }

LinkedList ll_copy(const LinkedList* src){ (void)src; LinkedList d; ll_init(&d); return d; }
LinkedList ll_move_from(LinkedList* src){ LinkedList d=*src; ll_init(src); return d; }
void ll_move_assign_from(LinkedList* dst, LinkedList* src){ (void)dst; (void)src; /* TODO */ }

