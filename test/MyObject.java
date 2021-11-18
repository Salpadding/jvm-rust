package test;

public class MyObject {
    public static int staticVar;
    public int instanceVar;
    public int instanceVar1;

    static {
        for(int i = 1; i <= 100; i++) {
            staticVar += i;
        }
    }

    public MyObject() {
    }

    public static void main(String[] args) {
        System.out.println(staticVar);
        MyObject myObj = new MyObject();
        myObj.staticVar = 1;
        int x = myObj.staticVar;

        myObj.instanceVar = 1;
        x = myObj.instanceVar;

        boolean b = myObj instanceof MyObject;
        boolean c = myObj instanceof java.lang.Object;
        Object d = (Object) myObj;
        MyObject e = (MyObject) d;

        long f = add(staticVar, 3000000000L);
        System.out.println(f);
    }

    public static long add(int a, long b) {
        return a + b;
    }
}