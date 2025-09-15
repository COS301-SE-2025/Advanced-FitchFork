#include "linked_list.h"
#include <stdlib.h>

void ll_init(LinkedList* l){ l->head=NULL; l->tail=NULL; l->size=0; }
int  ll_empty(const LinkedList* l){ return l->size==0; }
size_t ll_size(const LinkedList* l){ return l->size; }

void ll_clear(LinkedList* l){
    Node* n=l->head; while(n){ Node* nx=n->next; free(n); n=nx; }
    l->head=l->tail=NULL; l->size=0;
}

void ll_push_front(LinkedList* l, int v){
    Node* n=(Node*)malloc(sizeof(Node)); n->value=v; n->next=l->head; l->head=n; if(!l->tail) l->tail=n; l->size++;
}
void ll_push_back(LinkedList* l, int v){
    Node* n=(Node*)malloc(sizeof(Node)); n->value=v; n->next=NULL;
    if(!l->tail){ l->head=l->tail=n; }
    else { l->tail->next=n; l->tail=n; }
    l->size++;
}

int ll_pop_front(LinkedList* l, int* out){
    if(!l->head) return 0; Node* n=l->head; l->head=n->next; if(!l->head) l->tail=NULL; if(out) *out=n->value; free(n); l->size--; return 1;
}

int ll_front(const LinkedList* l, int* out){ if(!l->head) return 0; if(out) *out=l->head->value; return 1; }
int ll_back(const LinkedList* l, int* out){ if(!l->tail) return 0; if(out) *out=l->tail->value; return 1; }

int ll_insert(LinkedList* l, size_t index, int value){
    if(index>l->size) return 0; if(index==0){ ll_push_front(l,value); return 1; } if(index==l->size){ ll_push_back(l,value); return 1; }
    Node* prev=l->head; for(size_t i=0;i<index-1;i++) prev=prev->next;
    Node* n=(Node*)malloc(sizeof(Node)); n->value=value; n->next=prev->next; prev->next=n; l->size++; return 1;
}

int ll_erase(LinkedList* l, size_t index){
    if(index>=l->size) return 0; if(index==0){ int tmp; return ll_pop_front(l,&tmp); }
    Node* prev=l->head; for(size_t i=0;i<index-1;i++) prev=prev->next; Node* victim=prev->next; prev->next=victim->next; if(victim==l->tail) l->tail=prev; free(victim); l->size--; return 1;
}

LinkedList ll_copy(const LinkedList* src){
    LinkedList dst; ll_init(&dst); for(Node* n=src->head;n;n=n->next) ll_push_back(&dst, n->value); return dst;
}

LinkedList ll_move_from(LinkedList* src){ LinkedList dst=*src; ll_init(src); return dst; }

void ll_move_assign_from(LinkedList* dst, LinkedList* src){ ll_clear(dst); *dst=*src; ll_init(src); }

