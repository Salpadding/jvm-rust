package test;

public class MyObject {
    public static int staticVar;
    public int instanceVar;
    public int instanceVar1;

    public MyObject() {
        this.instanceVar = 1111;
        this.instanceVar1 = 1111 * 45;
    }

    public static void main(String[] args) {
        int u = 32768;
        MyObject myObj = new MyObject();
        myObj.staticVar = 1;
        int x = myObj.staticVar;

        myObj.instanceVar = 1;
        x = myObj.instanceVar;

        boolean b = myObj instanceof MyObject;
        boolean c = myObj instanceof java.lang.Object;
        Object d = (Object) myObj;
        MyObject e = (MyObject) d;

        long f = add(u, 3000000000L);
    }

    public static long add(int a, long b) {
        return a + b;
    }
}