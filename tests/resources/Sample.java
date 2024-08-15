import java.util.Map;
import java.util.HashMap;

public class Sample {
	
	interface MyInterface {
		MyAbstractClass getMyAbstractClass();
	}
	
	public static abstract class MyAbstractClass {
		private String stringField;
		private Map<Long, String> mapLongStringField;
	
		protected MyAbstractClass() {
			stringField = "string";
			mapLongStringField = new HashMap<>();
		}
	
		public String getStringField() {
			return stringField;
		}
	}
	
	public static class MyClass extends MyAbstractClass implements MyInterface {
		public MyClass() {
			super();
		}
	
		public MyAbstractClass getMyAbstractClass() {
			return this;
		}
	
		public long doSomething(int count) {
			int sum = 0;
			for (int i = 0; i < count; i++) {
				sum += i;
				System.out.println("sum is " + sum);
			}
	
			return sum;
	
		}
	
	}

	public static void main(String[] args) {
		Sample.MyClass mc = new Sample.MyClass();
		mc.doSomething(10);
	}
}
