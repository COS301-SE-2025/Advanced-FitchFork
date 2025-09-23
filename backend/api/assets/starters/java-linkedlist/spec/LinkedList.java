import java.util.*;

public class LinkedList {
	private static class Node {
		int value;
		Node next;

		Node(int v) {
			value = v;
		}
	}

	public static class IntBox {
		public int value;
	}

	private Node head, tail;
	private int size;

	public LinkedList() {
	}

	public LinkedList(LinkedList other) {
		/* TODO: copy from other */ }

	public static LinkedList moveFrom(LinkedList src) {
		// TODO: transfer ownership from src to a new list
		LinkedList dst = new LinkedList();
		return dst;
	}

	public void moveAssignFrom(LinkedList src) {
		if (this == src)
			return;
		// TODO: clear this and steal links from src
	}

	public boolean isEmpty() {
		return size == 0;
	}

	public int size() {
		return size;
	}

	public void clear() {
		// TODO: delete nodes
	}

	public void pushFront(int v) {
		// TODO
	}

	public void pushBack(int v) {
		// TODO
	}

	public boolean popFront(IntBox out) {
		// TODO
		return false;
	}

	public int front() {
		if (head == null)
			throw new IllegalStateException("empty");
		return head.value;
	}

	public int back() {
		if (tail == null)
			throw new IllegalStateException("empty");
		return tail.value;
	}

	public boolean insert(int index, int value) {
		// TODO
		return false;
	}

	public boolean erase(int index) {
		// TODO
		return false;
	}

	public List<Integer> toList() {
		ArrayList<Integer> out = new ArrayList<>(size);
		// TODO
		return out;
	}
}
