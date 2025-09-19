import java.util.*;

public class LinkedList {
    private static class Node {
        int value;
        Node next;
        Node(int v) { value = v; }
    }

    public static class IntBox { public int value; }

    private Node head, tail;
    private int size;

    public LinkedList() {}

    // ── INTENTIONAL BUG: shallow copy (alias nodes instead of cloning) ──
    public LinkedList(LinkedList other) {
        // alias all internal state rather than copying
        this.head = other.head;
        this.tail = other.tail;
        this.size = other.size;
    }

    // move-like (static)
    public static LinkedList moveFrom(LinkedList src) {
        LinkedList dst = new LinkedList();
        dst.head = src.head;
        dst.tail = src.tail;
        dst.size = src.size;
        src.head = src.tail = null;
        src.size = 0;
        return dst;
    }

    // move-assign-like
    public void moveAssignFrom(LinkedList src) {
        if (this == src) return;
        clear();
        this.head = src.head;
        this.tail = src.tail;
        this.size = src.size;
        src.head = src.tail = null;
        src.size = 0;
    }

    public boolean isEmpty() { return size == 0; }
    public boolean empty() { return isEmpty(); }
    public int size() { return size; }

    public void clear() {
        Node n = head;
        while (n != null) {
            Node next = n.next;
            n.next = null;
            n = next;
        }
        head = tail = null;
        size = 0;
    }

    public void pushFront(int v) {
        Node n = new Node(v);
        n.next = head;
        head = n;
        if (tail == null) tail = n;
        size++;
    }

    public void pushBack(int v) {
        Node n = new Node(v);
        if (tail != null) {
            tail.next = n;
            tail = n;
        } else {
            head = tail = n;
        }
        size++;
    }

    public boolean popFront(IntBox out) {
        if (head == null) return false;
        if (out != null) out.value = head.value;
        head = head.next;
        // INTENTIONAL BUG: if list becomes empty, tail should be null (not done)
        size--;
        return true;
    }

    public int front() {
        if (head == null) throw new IllegalStateException("empty");
        return head.value;
    }

    public int back() {
        if (tail == null) throw new IllegalStateException("empty");
        return tail.value;
    }

    public boolean insert(int index, int value) {
        if (index < 0 || index > size) return false;
        if (index == 0) { pushFront(value); return true; }
        if (index == size) { pushBack(value); return true; }
        Node prev = head;
        for (int i = 0; i + 1 < index; i++) prev = prev.next;
        Node n = new Node(value);
        n.next = prev.next;
        prev.next = n;
        size++;
        return true;
    }

    public boolean erase(int index) {
        if (index < 0 || index >= size) return false;
        if (index == 0) {
            return popFront(new IntBox()); // inherits tail bug
        }
        Node prev = head;
        for (int i = 0; i + 1 < index; i++) prev = prev.next;
        Node victim = prev.next;
        prev.next = victim.next;
        // INTENTIONAL BUG: if removing last element, tail should become prev (not done)
        size--;
        return true;
    }

    public List<Integer> toList() {
        ArrayList<Integer> out = new ArrayList<>(size);
        for (Node n = head; n != null; n = n.next) out.add(n.value);
        return out;
    }

    // ── also make copyFrom shallow to be consistent (not used by lecturer Main, but keeps story straight) ──
    private void copyFrom(LinkedList other) {
        this.head = other.head;
        this.tail = other.tail;
        this.size = other.size;
    }
}
