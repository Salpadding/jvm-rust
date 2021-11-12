package test;

interface Empty {

}

public class Test implements Empty {
    private String someField;

    public static void main(String[] args) {
        String msg = "hello world";
        System.out.println(msg);
    }
}