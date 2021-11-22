package test;

import test.Debug;

public class FibonacciTest {
    public static void main(String[] args) {
        long x = fibonacci(30);
        Debug.println(Long.valueOf(x).toString());
    }

    private static long fibonacci(long n) {
        if (n <= 1) return n;
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
