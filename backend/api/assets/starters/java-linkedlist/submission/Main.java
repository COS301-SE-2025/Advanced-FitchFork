public class Main {
	public static void main(String[] args) {
		LinkedList lst = new LinkedList();
		lst.pushBack(10);
		lst.pushBack(20);
		lst.pushBack(30);

		LinkedList.IntWrapper out = new LinkedList.IntWrapper();
		lst.popFront(out);

		lst.insert(1, 111);
		lst.erase(2);

		System.out.println("List = " + lst.toList());
		System.out.println("size = " + lst.size());
	}
}
