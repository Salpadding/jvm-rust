na!(
    ClassReg,
    "java/lang/Class",
    "registerNatives",
    "()V",
    th,
    f,
    { reg!(th.registry, N0, N1, N2) }
);

na!(
    N0,
    "java/lang/Class",
    "getPrimitiveClass",
    "(Ljava/lang/String;)Ljava/lang/Class;",
    th,
    f,
    {
        let s = f.this().as_utf8();
        let cl = f.heap.loader.load(&s);
        f.stack.push_obj(cl.j_class)
    }
);

na!(
    N1,
    "java/lang/Class",
    "desiredAssertionStatus0",
    "(Ljava/lang/Class;)Z",
    th,
    f,
    { f.stack.push_u32(0) }
);

na!(
    N2,
    "java/lang/Class",
    "getName0",
    "()Ljava/lang/String;",
    th,
    f,
    {
        let cl = f.this().class;
        let js = f.heap.new_jstr(&cl.name);
        f.stack.push_obj(js)
    }
);
