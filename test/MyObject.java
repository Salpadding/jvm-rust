package test;

public class MyObject {
    public static int staticVar;
    public int instanceVar;

    public static void main(String[] args) {
        MyObject myObj = new MyObject();
        myObj.staticVar = 1;
        int x = myObj.staticVar;

        myObj.instanceVar = 1;
        x = myObj.instanceVar;
    }
}