use std::mem::size_of;

na!(
    UnsafeReg,
    "sun/misc/Unsafe",
    "registerNatives",
    "()V",
    th,
    f,
    {
        reg!(th.registry, N0, N1, N2);
    }
);

na!(
    N0,
    "sun/misc/Unsafe",
    "arrayBaseOffset",
    "(Ljava/lang/Class;)I",
    th,
    f,
    { f.stack.push_u32(0) }
);

na!(
    N1,
    "sun/misc/Unsafe",
    "arrayIndexScale",
    "(Ljava/lang/Class;)I",
    th,
    f,
    { f.stack.push_u32(0) }
);

na!(N2, "sun/misc/Unsafe", "addressSize", "()I", th, f, {
    f.stack.push_u32(size_of::<usize>() as u32)
});
