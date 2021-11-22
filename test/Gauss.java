package test;

import test.Debug;

public class Gauss {
    public static final int MAX = 100;

    public static void main(String[] args) {
        int sum = 0;
        for(int i = 0; i <= 100; i++) {
            sum += i;
        }

        Debug.println(sum);
    }
}