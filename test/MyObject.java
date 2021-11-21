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
        String a = "124";
        String b = "124";

        boolean c = a.equals(b);
    }

    public static long add(int a, long b) {
        return a + b;
    }
}