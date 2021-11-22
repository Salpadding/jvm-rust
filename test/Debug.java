package test;

// used for debug
public class Debug {
    public static native void registerNatives();

    static {
        registerNatives();
    }

    public static native void print(String s);
    public static native void print(int s);
    public static native void print(long s);
    public static native void print(float s);
    public static native void print(double s);

    public static void println(String s){
        print(s);
        print("\n");
    }
    public static void println(int s){
        print(s);
        print("\n");
    }
    public static void println(long s){
        print(s);
        print("\n");
    }
    public static void println(float s){
        print(s);
        print("\n");
    }
    public static void println(double s){
        print(s);
        print("\n");
    }

    public static void main(String[] args) {
        println("hello world");
        println(1000L);
        println(-1000L);
    }
}