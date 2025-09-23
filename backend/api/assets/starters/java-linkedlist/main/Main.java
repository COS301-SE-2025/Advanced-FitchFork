import java.util.*;

public class Main {
	// ExecutionConfig.default_deliminator() => "&-=-&"
	private static final String DELIM = "&-=-&";

	private static void printSection(String name) {
		System.out.println(DELIM + " " + name);
	}

	private static void printList(LinkedList lst, String label) {
		if (!label.isEmpty())
			System.out.print(label + ": ");
		List<Integer> vs = lst.toList();
		System.out.print("[");
		for (int i = 0; i < vs.size(); i++) {
			if (i > 0)
				System.out.print(" ");
			System.out.print(vs.get(i));
		}
		System.out.println("] size=" + lst.size());
	}

	// ───────── tasks ─────────
	private static void task1_basic_ops() {
		printSection("start-task1");

		LinkedList lst = new LinkedList();
		printSection("empty-list");
		System.out.println("empty=" + lst.isEmpty() + " size=" + lst.size());

		printSection("push_front_back");
		lst.pushFront(2);
		lst.pushBack(5);
		lst.pushFront(1);
		printList(lst, "after-push");

		printSection("front_back");
		System.out.println("front=" + lst.front() + " back=" + lst.back());

		printSection("pop_front");
		LinkedList.IntBox box = new LinkedList.IntBox();
		boolean ok = lst.popFront(box);
		System.out.println("ok=" + ok + " popped=" + (ok ? Integer.toString(box.value) : "N/A"));
		printList(lst, "after-pop");

		printSection("clear");
		lst.clear();
		System.out.println("empty=" + lst.isEmpty() + " size=" + lst.size());

		printSection("pop_last_then_push");
		LinkedList one = new LinkedList();
		one.pushBack(7); // [7]
		LinkedList.IntBox box2 = new LinkedList.IntBox();
		boolean ok2 = one.popFront(box2); // should make list empty *and* tail=null
		System.out.println("ok=" + ok2 + " popped=" + (ok2 ? Integer.toString(box2.value) : "N/A"));
		System.out.println("empty=" + one.isEmpty() + " size=" + one.size());
		one.pushBack(99); // if tail wasn’t nulled, head may stay null → broken list
		printList(one, "after-pop-last-then-push");
	}

	private static void task2_insert_erase() {
		printSection("start-task2");

		LinkedList lst = new LinkedList();
		for (int i = 1; i <= 5; i++)
			lst.pushBack(i); // [1 2 3 4 5]
		printList(lst, "seed");

		printSection("insert");
		System.out.println("ok=" + lst.insert(0, 100)); // [100 1 2 3 4 5]
		System.out.println("ok=" + lst.insert(3, 200)); // [100 1 2 200 3 4 5]
		System.out.println("ok=" + lst.insert(lst.size(), 300)); // append
		printList(lst, "after-insert");

		printSection("erase");
		System.out.println("ok=" + lst.erase(0)); // remove 100
		System.out.println("ok=" + lst.erase(2)); // remove 200 (now index 2)
		System.out.println("ok=" + lst.erase(lst.size() - 1)); // remove 300 (tail)
		printList(lst, "after-erase");

		printSection("erase-tail-then-push");
		boolean okTail = lst.erase(lst.size() - 1); // erase current tail
		System.out.println("ok=" + okTail);
		lst.pushBack(999); // if tail wasn’t fixed, linkage breaks
		printList(lst, "after-erase-tail-then-push");
	}

	private static void task3_copy_move() {
		printSection("start-task3");

		LinkedList a = new LinkedList();
		for (int i = 0; i < 4; i++)
			a.pushBack(i * 10); // [0 10 20 30]
		printList(a, "a");

		printSection("copy-ctor");
		LinkedList b = new LinkedList(a); // shallow copy here will alias nodes
		printList(b, "b");

		printSection("modify-original");
		a.pushBack(40); // a becomes [0 10 20 30 40]
		a.erase(1); // a becomes [0 20 30 40]
		printList(a, "a-after");

		// If student's copy-ctor was shallow, b will also have changed (or size/list
		// mismatch)
		printList(b, "b-unchanged");

		printSection("steal/move-sim");
		LinkedList c = LinkedList.moveFrom(a);
		printList(c, "c");
		printList(a, "a-moved-from");

		printSection("move-assign-sim");
		LinkedList d = new LinkedList();
		d.moveAssignFrom(c);
		printList(d, "d");
		printList(c, "c-moved-from");
	}

	public static void main(String[] args) {
		String which = (args.length >= 1) ? args[0] : "";
		if ("task1".equals(which)) {
			task1_basic_ops();
			return;
		}
		if ("task2".equals(which)) {
			task2_insert_erase();
			return;
		}
		if ("task3".equals(which)) {
			task3_copy_move();
			return;
		}
		task1_basic_ops();
		task2_insert_erase();
		task3_copy_move();
	}
}
