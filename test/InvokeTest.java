package test;

import test.Debug;

public class InvokeTest implements Runnable {
    public static void main(String[] args) {
        new InvokeTest().test();
    }

    public void test() {
        InvokeTest.staticMethod();
        InvokeTest test = new InvokeTest();
        test.instanceMethod();
        super.equals(null);
        this.run();
        ((Runnable) test).run();
        Debug.println(1000L);
    }

    public static void staticMethod() {

    }

    private void instanceMethod() {}

    @Override
    public void run() {

    }
}