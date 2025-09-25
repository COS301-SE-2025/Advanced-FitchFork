#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "linked_list.h"

#define DELIM "###"

static void section(const char *name) { printf(DELIM " %s\n", name); }

static void print_list(const LinkedList *lst, const char *label)
{
    if (label && *label)
        printf("%s: ", label);
    printf("[");
    const Node *n = lst->head;
    int first = 1;
    while (n)
    {
        if (!first)
            printf(" ");
        first = 0;
        printf("%d", n->value);
        n = n->next;
    }
    printf("] size=%zu\n", lst->size);
}

static void task1_basic_ops(void)
{
    section("start-task1");
    LinkedList lst;
    ll_init(&lst);

    section("empty-list");
    printf("empty=%s size=%zu\n", ll_empty(&lst) ? "true" : "false", ll_size(&lst));

    section("push_front_back");
    ll_push_front(&lst, 2);
    ll_push_back(&lst, 5);
    ll_push_front(&lst, 1);
    print_list(&lst, "after-push");

    section("front_back");
    int f = 0, b = 0;
    ll_front(&lst, &f);
    ll_back(&lst, &b);
    printf("front=%d back=%d\n", f, b);

    section("pop_front");
    int popped = 0;
    int ok = ll_pop_front(&lst, &popped);
    printf("ok=%s popped=%d\n", ok ? "true" : "false", ok ? popped : 0);
    print_list(&lst, "after-pop");

    section("clear");
    ll_clear(&lst);
    printf("empty=%s size=%zu\n", ll_empty(&lst) ? "true" : "false", ll_size(&lst));

    section("pop_last_then_push");
    LinkedList one;
    ll_init(&one);
    ll_push_back(&one, 7);
    int popped2 = 0;
    int ok2 = ll_pop_front(&one, &popped2);
    printf("ok=%s popped=%d\n", ok2 ? "true" : "false", ok2 ? popped2 : 0);
    printf("empty=%s size=%zu\n", ll_empty(&one) ? "true" : "false", ll_size(&one));
    ll_push_back(&one, 99);
    print_list(&one, "after-pop-last-then-push");
    ll_clear(&one);
    ll_clear(&lst);
}

static void task2_insert_erase(void)
{
    section("start-task2");
    LinkedList lst;
    ll_init(&lst);
    for (int i = 1; i <= 5; i++)
        ll_push_back(&lst, i);
    print_list(&lst, "seed");

    section("insert");
    printf("ok=%s\n", ll_insert(&lst, 0, 100) ? "true" : "false");
    printf("ok=%s\n", ll_insert(&lst, 3, 200) ? "true" : "false");
    printf("ok=%s\n", ll_insert(&lst, ll_size(&lst), 300) ? "true" : "false");
    print_list(&lst, "after-insert");

    section("erase");
    printf("ok=%s\n", ll_erase(&lst, 0) ? "true" : "false");
    printf("ok=%s\n", ll_erase(&lst, 2) ? "true" : "false");
    printf("ok=%s\n", ll_erase(&lst, ll_size(&lst) - 1) ? "true" : "false");
    print_list(&lst, "after-erase");

    section("erase-tail-then-push");
    int okTail = ll_erase(&lst, ll_size(&lst) - 1);
    printf("ok=%s\n", okTail ? "true" : "false");
    ll_push_back(&lst, 999);
    print_list(&lst, "after-erase-tail-then-push");
    ll_clear(&lst);
}

static void task3_copy_move(void)
{
    section("start-task3");
    LinkedList a;
    ll_init(&a);
    for (int i = 0; i < 4; i++)
        ll_push_back(&a, i * 10);
    print_list(&a, "a");

    section("copy-ctor");
    LinkedList b = ll_copy(&a);
    print_list(&b, "b");

    section("modify-original");
    ll_push_back(&a, 40);
    (void)ll_erase(&a, 1);
    print_list(&a, "a-after");
    print_list(&b, "b-unchanged");

    section("steal/move-sim");
    LinkedList c = ll_move_from(&a);
    print_list(&c, "c");
    print_list(&a, "a-moved-from");

    section("move-assign-sim");
    LinkedList d;
    ll_init(&d);
    ll_move_assign_from(&d, &c);
    print_list(&d, "d");
    print_list(&c, "c-moved-from");

    ll_clear(&a);
    ll_clear(&b);
    ll_clear(&c);
    ll_clear(&d);
}

int main(int argc, char **argv)
{
    const char *which = argc >= 2 ? argv[1] : "";
    if (strcmp(which, "task1") == 0)
    {
        task1_basic_ops();
        return 0;
    }
    if (strcmp(which, "task2") == 0)
    {
        task2_insert_erase();
        return 0;
    }
    if (strcmp(which, "task3") == 0)
    {
        task3_copy_move();
        return 0;
    }
    task1_basic_ops();
    task2_insert_erase();
    task3_copy_move();
    return 0;
}
