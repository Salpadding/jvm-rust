use crate::runtime::misc::Slots;

na!(
    JLOReg,
    "java/lang/Object",
    "registerNatives",
    "()V",
    th,
    f,
    {
        reg!(f.registry, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, N10, N11, N12, N13);
    }
);

na!(
    N0,
    "java/lang/Object",
    "getClass",
    "()Ljava/lang/Class;",
    th,
    f,
    {
        let ths = f.this();
        f.stack.push_obj(ths.class.j_class);
    }
);

na!(N1, "java/lang/Object", "hashCode", "()I", th, f, {
    let ths = f.this();
    f.stack.push_u32(ths.ptr() as u32)
});

na!(N2, "java/lang/Float", "floatToRawIntBits", "(F)I", th, f, {
    f.stack.push_u32(f.local_vars[0] as u32)
});

na!(
    N3,
    "java/lang/Double",
    "doubleToRawLongBits",
    "(D)J",
    th,
    f,
    { f.stack.push_u64(f.local_vars.get_u64(0)) }
);

na!(N4, "java/lang/Double", "longBitsToDouble", "(J)D", th, f, {
    f.stack.push_u64(f.local_vars.get_u64(0))
});

na!(N5, "java/io/FileOutputStream", "initIDs", "()V", th, f, {});
na!(N6, "java/io/FileInputStream", "initIDs", "()V", th, f, {});
na!(N7, "java/io/FileDescriptor", "initIDs", "()V", th, f, {});
na!(N8, "java/io/FileDescriptor", "set", "(I)J", th, f, {
    f.stack.push_u64(f.local_vars[0]);
});

na!(
    N9,
    "sun/reflect/Reflection",
    "getCallerClass",
    "()Ljava/lang/Class;",
    th,
    f,
    {
        let caller_frame = th.back_frame(3);
        let caller_class = caller_frame.class.j_class;
        f.stack.push_obj(caller_class)
    }
);

macro_rules! ac {
    ($th: ident, $f: ident) => {{
        let this = $f.this();

        $th.invoke_obj(
            this.get_mut(),
            "run",
            "()Ljava/lang/Object;",
            &[this.ptr() as u64],
            false,
        );
    }};
}
na!(
    N10,
    "java/security/AccessController",
    "doPrivileged",
    "(Ljava/security/PrivilegedExceptionAction;)Ljava/lang/Object;",
    th,
    f,
    { ac!(th, f) }
);

na!(
    N11,
    "java/security/AccessController",
    "doPrivileged",
    "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;",
    th,
    f,
    { ac!(th, f) }
);

na!(
    N12,
    "java/security/AccessController",
    "getStackAccessControlContext",
    "()Ljava/security/AccessControlContext;",
    th,
    f,
    { f.stack.push_null() }
);

na!(
    N13,
    "java/lang/String",
    "intern",
    "()Ljava/lang/String;",
    th,
    f,
    {
        let this = f.this();
        println!("String.intern {} ", this.jstring());
        let o = f.heap.new_jstr(&this.jstring());
        f.stack.push_obj(o)
    }
);
