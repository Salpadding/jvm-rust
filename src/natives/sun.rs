use crate::heap::class::Object;
use rp::Rp;
use std::mem::size_of;

na!(
    UnsafeReg,
    "sun/misc/Unsafe",
    "registerNatives",
    "()V",
    th,
    f,
    {
        reg!(th.registry, N0, N1, N2, N3, N4);
    }
);

na!(
    N0,
    "sun/misc/Unsafe",
    "arrayBaseOffset",
    "(Ljava/lang/Class;)I",
    th,
    f,
    { f.push_u32(0) }
);

na!(
    N1,
    "sun/misc/Unsafe",
    "arrayIndexScale",
    "(Ljava/lang/Class;)I",
    th,
    f,
    { f.push_u32(0) }
);

na!(N2, "sun/misc/Unsafe", "addressSize", "()I", th, f, {
    f.push_u32(size_of::<usize>() as u32)
});

na!(
    N3,
    "sun/misc/Unsafe",
    "objectFieldOffset",
    "(Ljava/lang/reflect/Field;)J",
    th,
    f,
    {
        let field: Rp<Object> = (f.local_vars()[1] as usize).into();
        let slot = field.get_field("slot");
        f.push_u64(slot)
    }
);

na!(
    N4,
    "sun/misc/Unsafe",
    "compareAndSwapObject",
    "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
    th,
    f,
    { panic!("cas") }
);
