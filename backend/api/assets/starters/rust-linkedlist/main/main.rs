mod linked_list;

use linked_list::LinkedList;
use std::env;

// Keep delimiter consistent with other languages
const DELIM: &str = "###"; // matches ExecutionConfig.default_deliminator()

fn section(name: &str) {
    println!("{} {}", DELIM, name);
}

fn print_list<T: std::fmt::Display>(lst: &LinkedList<T>, label: &str) {
    if !label.is_empty() {
        print!("{}: ", label);
    }
    print!("[");
    let mut first = true;
    for v in lst.to_vec() {
        if !first {
            print!(" ");
        }
        first = false;
        print!("{}", v);
    }
    println!("] size={}", lst.len());
}

// ───────── tasks ─────────
fn task1_basic_ops() {
    section("start-task1");

    let mut lst = LinkedList::new();
    section("empty-list");
    println!("empty={} size={}", lst.is_empty(), lst.len());

    section("push_front_back");
    lst.push_front(2);
    lst.push_back(5);
    lst.push_front(1);
    print_list(&lst, "after-push");

    section("front_back");
    let f = lst.front().copied().unwrap_or_default();
    let b = lst.back().copied().unwrap_or_default();
    println!("front={} back={}", f, b);

    section("pop_front");
    let popped = lst.pop_front();
    println!(
        "ok={} popped={}",
        popped.is_some(),
        popped.unwrap_or_default()
    );
    print_list(&lst, "after-pop");

    section("clear");
    lst.clear();
    println!("empty={} size={}", lst.is_empty(), lst.len());

    section("pop_last_then_push");
    let mut one = LinkedList::new();
    one.push_back(7);
    let last = one.pop_front();
    println!("ok={} popped={}", last.is_some(), last.unwrap_or_default());
    println!("empty={} size={}", one.is_empty(), one.len());
    one.push_back(99);
    print_list(&one, "after-pop-last-then-push");
}

fn task2_insert_erase() {
    section("start-task2");

    let mut lst = LinkedList::new();
    for i in 1..=5 {
        lst.push_back(i);
    }
    print_list(&lst, "seed");

    section("insert");
    println!("ok={}", lst.insert_at(0, 100).is_ok());
    println!("ok={}", lst.insert_at(3, 200).is_ok());
    println!("ok={}", lst.insert_at(lst.len(), 300).is_ok());
    print_list(&lst, "after-insert");

    section("erase");
    println!("ok={}", lst.remove_at(0).is_ok());
    println!("ok={}", lst.remove_at(2).is_ok());
    println!("ok={}", lst.remove_at(lst.len() - 1).is_ok());
    print_list(&lst, "after-erase");

    section("erase-tail-then-push");
    let ok_tail = lst.remove_at(lst.len() - 1).is_ok();
    println!("ok={}", ok_tail);
    lst.push_back(999);
    print_list(&lst, "after-erase-tail-then-push");
}

fn task3_copy_move() {
    section("start-task3");

    let mut a = LinkedList::new();
    for i in 0..4 {
        a.push_back(i * 10);
    }
    print_list(&a, "a");

    section("copy-ctor");
    let b = a.clone();
    print_list(&b, "b");

    section("modify-original");
    a.push_back(40);
    let _ = a.remove_at(1);
    print_list(&a, "a-after");
    print_list(&b, "b-unchanged");

    section("steal/move-sim");
    let mut a2 = LinkedList::move_from(&mut a);
    print_list(&a2, "c");
    print_list(&a, "a-moved-from");

    section("move-assign-sim");
    let mut d = LinkedList::new();
    d.move_assign_from(&mut a2);
    print_list(&d, "d");
    print_list(&a2, "c-moved-from");
}

fn main() {
    let arg = env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "task1" => task1_basic_ops(),
        "task2" => task2_insert_erase(),
        "task3" => task3_copy_move(),
        _ => {
            task1_basic_ops();
            task2_insert_erase();
            task3_copy_move();
        }
    }
}
